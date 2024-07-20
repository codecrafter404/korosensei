use std::path::PathBuf;

use chrono::{DateTime, Utc};
use color_eyre::eyre::{Context, OptionExt as _};
use itertools::Itertools;

use crate::utils::{
    config::{Config, TranscriptionConfig},
    git,
};

/// Discovers all .link files which have to be transcribed
/// returns absolute paths
pub(crate) fn discover_files(conf: &Config) -> color_eyre::Result<Vec<PathBuf>> {
    let transcription_conf = conf
        .transcription
        .clone()
        .ok_or_eyre("Expected transcription configuration to be loaded")?;
    let _ = git::check_out_create_branch(&transcription_conf.git_source_branch, &conf)?;

    let source_path = conf.git_directory.join(
        transcription_conf
            .git_source_path
            .strip_prefix("/")
            .unwrap_or(&transcription_conf.git_source_path),
    );

    let mut link_files = Vec::new();

    for file in std::fs::read_dir(&source_path)? {
        let dir_entry = match file {
            Ok(x) => x,
            Err(why) => {
                log::error!("Skipped dir entry: {:?}", why);
                continue;
            }
        };
        if dir_entry.file_type().is_ok_and(|x| x.is_dir()) {
            continue;
        }
        if let Some(filename) = dir_entry.file_name().to_str() {
            if !filename.ends_with(".link") {
                log::warn!("Skipping non .link file: {:?}", dir_entry.path());
                continue;
            }
            let name_without_extension = filename
                .strip_suffix(".link")
                .expect("Infallible")
                .to_owned();
            debug_assert!(name_without_extension.split(".").last().is_some()); // should be mp3/wav/ or any other configured audio file format

            link_files.push((dir_entry.path(), name_without_extension));
        }
    }

    let _ = git::check_out_create_branch(&transcription_conf.git_target_branch, &conf)?;

    let mut transcribed_files = vec![];

    let target_path = conf.git_directory.join(
        transcription_conf
            .transcription_target_path
            .strip_prefix("/")
            .unwrap_or(&transcription_conf.transcription_target_path),
    );
    std::fs::create_dir_all(target_path.clone())?;

    for file in std::fs::read_dir(&target_path)? {
        let dir_entry = match file {
            Ok(x) => x,
            Err(why) => {
                log::error!("Skipped dir entry: {:?}", why);
                continue;
            }
        };
        if dir_entry.file_type().is_ok_and(|x| x.is_dir()) {
            continue;
        }
        if let Some(filename) = dir_entry.file_name().to_str() {
            if !filename.ends_with(".transcript.md") {
                log::warn!("Skipping non .transcript.md file: {:?}", dir_entry.path());
                continue;
            }
            let name_without_extension = filename
                .strip_suffix(".transcript.md")
                .expect("Infallible")
                .to_owned();
            debug_assert!(name_without_extension.split(".").last().is_some()); // should be mp3/wav/ or any other configured audio file format
            transcribed_files.push((dir_entry.path(), name_without_extension))
        }
    }

    let links_to_transcribe = link_files
        .into_iter()
        .filter(|(_, name)| transcribed_files.iter().all(|(_, b)| b != name))
        .map(|x| x.0)
        .collect_vec();

    Ok(links_to_transcribe)
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlamedFile {
    pub file: PathBuf,
    pub blame: Vec<BlameResult>,
}
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
    let files = collect_files_to_blame(base_path).wrap_err("Failed to collect files for blame")?;

    for file in files {
        match blame_file(file.clone()) {
            Ok(x) => {}
            Err(why) => {}
        }
    }

    unimplemented!()
}
fn collect_files_to_blame(path: PathBuf) -> color_eyre::Result<Vec<PathBuf>> {
    unimplemented!()
}
fn blame_file(path: PathBuf) -> color_eyre::Result<BlamedFile> {
    unimplemented!()
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlameResult {
    /// Utc, raw
    pub time: DateTime<Utc>,
    /// 0-indexed
    pub line: usize,
}

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
    let res = parse_git_blame(&input).unwrap();
    assert_eq!(res, expected);
}
