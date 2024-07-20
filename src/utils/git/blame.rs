use std::path::PathBuf;

use chrono::{DateTime, Utc};
use color_eyre::eyre::{Context as _, OptionExt as _};
use itertools::Itertools as _;

use crate::utils::config::Config;
use crate::utils::git;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlamedFile {
    pub file: PathBuf,
    pub blame: Vec<BlameResult>,
}
impl BlamedFile {
    /// returns (_, errored)
    /// errored...      represents if one file has failed blaming
    pub fn blame_all(conf: &Config) -> color_eyre::Result<(Vec<BlamedFile>, bool)> {
        let transcription_config = conf
            .transcription
            .clone()
            .ok_or_eyre("Expected transcription config to be initialized")?;
        let base_path = conf.git_directory.join(
            transcription_config
                .transcription_script_search_path
                .strip_prefix("/")
                .unwrap_or(&transcription_config.transcription_script_search_path),
        );
        let files = BlamedFile::collect_files_to_blame(base_path)
            .wrap_err("Failed to collect files for blame")?;

        let mut res = Vec::new();
        let mut errored = false;
        for file in files {
            match BlamedFile::blame_file(file.clone(), &conf) {
                Ok(x) => {
                    res.push(x);
                }
                Err(why) => {
                    errored = true;
                    log::error!("Failed to blame file {:?}: {:?}", file, why);
                }
            }
        }

        Ok((res, errored))
    }
    /// returnes absolute paths
    fn collect_files_to_blame(path: PathBuf) -> color_eyre::Result<Vec<PathBuf>> {
        let walker = walkdir::WalkDir::new(path).into_iter().filter_entry(|e| {
            e.file_name()
                .to_str()
                .is_some_and(|x| x.ends_with(".md") && !x.ends_with(".transcription.md"))
                && !e.path_is_symlink()
        });
        let mut res = Vec::new();
        for entry in walker {
            match entry {
                Ok(entry) => res.push(entry.into_path()),
                Err(why) => {
                    log::error!("Failed to walk dir entry: {:?}", why);
                }
            }
        }
        Ok(res)
    }
    /// git blame --line-porcelain <path>
    fn blame_file(path: PathBuf, conf: &Config) -> color_eyre::Result<BlamedFile> {
        let res = git::git_command_wrapper(
            &[
                "blame",
                "--line-porcelain",
                path.to_str().ok_or_eyre("expected to get parsable path")?,
            ],
            &conf.git_directory,
            conf,
        )?;
        git::wrap_git_command_error(&res)?;
        let res = BlameResult::parse_git_blame(&res.std_out)?;
        Ok(BlamedFile {
            file: path,
            blame: res,
        })
    }

    pub fn to_correlating_file(
        &self,
        conf: &Config,
        date: DateTime<Utc>,
    ) -> color_eyre::Result<crate::jobs::transcription::markdown::CorrelatingFile> {
        unimplemented!()
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlameResult {
    /// Utc, raw
    pub time: DateTime<Utc>,
    /// 0-indexed
    pub line: usize,
}

impl BlameResult {
    /// parses each line for git blame
    /// expects to have the result of git blame --line-porcelain
    fn parse_git_blame(blame: &str) -> color_eyre::Result<Vec<BlameResult>> {
        let to_strip = vec![
            "author ",
            "author-mail ",
            "author-time ",
            "author-tz ",
            "committer ",
            "committer-mail ",
            "committer-tz ",
            "summary ",
            "filename ",
        ];

        let lines = blame
            .split("\n")
            .into_iter()
            .filter(|x| to_strip.iter().all(|y| x.strip_prefix(y).is_none())) // this extracts all line that don't start with any item in to_strip
            .collect_vec();

        let blame = lines.join("\n");
        println!("{:#?}", blame);

        let regex = regex::RegexBuilder::new(
            r"^(?:[\da-f]{40}) (?:\d{1,}) (\d{1,})(?: \d{1,})?\ncommitter-time (\d{1,})$",
        )
        .multi_line(true)
        .build()?;
        let mut res = vec![];
        for (_, [line, time]) in regex.captures_iter(&blame).map(|x| x.extract()) {
            let line = line
                .parse::<usize>()
                .wrap_err("Failed to parse blame line")?
                - 1;
            let time = time
                .parse::<i64>()
                .wrap_err("Failed to parse committer-time")?;
            let time = DateTime::from_timestamp(time, 0)
                .ok_or_eyre("Failed to parse committer-time -> DateTime<Utc>")?;
            res.push(BlameResult { time, line });
        }

        Ok(res)
    }
}

#[test]
fn test_parse_git_blame() {
    let input = "\
d62b144c8ac3c0a942c1a5fa703c69967612e98e 1 1 208
author Koro-sensei
author-mail <koro-sensei@ansatsu-anime.com>
author-time 1721484187
author-tz +0200
committer Codecrafter_404
committer-mail <codecrafter404@github.com>
committer-time 1721484369
committer-tz +0200
summary transcribed: test.mp3.link
filename attachements/test.mp3.transcript.md
        # Transcript '19.07.2024 20:51'
d62b144c8ac3c0a942c1a5fa703c69967612e98e 2 2
author Koro-sensei
author-mail <koro-sensei@ansatsu-anime.com>
author-time 1721484342
author-tz +0200
committer Codecrafter_404
committer-mail <codecrafter404@github.com>
committer-time 1721484342
committer-tz +0200
summary transcribed: test.mp3.link
filename attachements/test.mp3.transcript.md

d62b144c8ac3c0a942c1a5fa703c69967612e98e 3 3
author Koro-sensei
author-mail <koro-sensei@ansatsu-anime.com>
author-time 172140000
author-tz +0200
committer Codecrafter_404
committer-mail <codecrafter404@github.com>
committer-time 1721480000
committer-tz +0200
summary transcribed: test.mp3.link
filename attachements/test.mp3.transcript.md
        > _Links
";
    let expected = vec![
        BlameResult {
            time: DateTime::from_timestamp(1721484369, 0).unwrap(),
            line: 0,
        },
        BlameResult {
            time: DateTime::from_timestamp(1721484342, 0).unwrap(),
            line: 1,
        },
        BlameResult {
            time: DateTime::from_timestamp(1721480000, 0).unwrap(),
            line: 2,
        },
    ];
    let res = BlameResult::parse_git_blame(&input).unwrap();
    assert_eq!(res, expected);
}
