use std::ops::{Deref, Sub as _};
use std::path::PathBuf;
use std::str::FromStr;

use chrono::{Date, DateTime, Utc};
use color_eyre::eyre::{eyre, Context, OptionExt};

use crate::utils::config::Config;
use crate::utils::git::{self, git_command_wrapper};
use itertools::Itertools;

pub(crate) struct CorrelatingFile {
    path: PathBuf,
    headlines: Vec<u32>,
}

pub(crate) async fn discorver_correlating_files(
    time: DateTime<Utc>,
    config: &Config,
) -> color_eyre::Result<Vec<CorrelatingFile>> {
    //TODO::
    //1. search all the files that have been changed the timerange (git log? -> find commits +/- n minutes)
    //2. extract only the affected headlines (affected headline = headline changed / new_headline / something under the headline has changed)
    //return the correlating files array

    let commits = get_related_commits(&config, time.clone())?;
    for commit in commits {
        let files = diff_commit(&commit, config)?;
    }

    unimplemented!();
}

fn diff_commit(commit_id: &str, config: &Config) -> color_eyre::Result<Vec<(PathBuf, Vec<i32>)>> {
    let res = git::git_command_wrapper(
        &["diff", "-p", &format!("{}~1", commit_id), commit_id],
        &config.git_directory,
        config,
    )?;
    git::wrap_git_command_error(&res)?;
    let patches = patch::Patch::from_multiple(&res.std_out);
    if let Ok(patches) = patches {
        let mut lines = get_changed_lines(&patches)?;
        lines.dedup();

        return Ok(lines);
    }
    return Err(eyre!("Failed to parse patches {:?}", patches));
}
fn get_changed_lines(patches: &Vec<patch::Patch>) -> color_eyre::Result<Vec<(PathBuf, i32)>> {
    let mut file_changes: Vec<(PathBuf, i32)> = vec![];
    for patch in patches {
        let path = patch.new.path.to_string().trim().to_owned();
        if path == "/dev/null" {
            continue;
        }
        let path = path
            .strip_prefix("b/")
            .ok_or_eyre("Expected git patch to have a b/ path prefix")?;
        let path = PathBuf::from_str(&path)?;

        for hunk in &patch.hunks {
            let mut current_line = (hunk.new_range.start - 1) as i32;
            current_line -= 1; // starting point
            for line in &hunk.lines {
                match line {
                    patch::Line::Add(_) => {
                        current_line += 1;
                        file_changes.push((path.clone(), current_line));
                    }
                    patch::Line::Remove(_) => {}
                    patch::Line::Context(_) => current_line += 1,
                }
            }
        }
    }

    return Ok(file_changes);
}
fn get_related_markdown_headings(
    line: u64,
    content: &str,
    include_parents: bool,
) -> color_eyre::Result<Vec<u64>> {
    let lines = content.split("\n").collect::<Vec<_>>();
    if line as usize >= lines.len() {
        return Err(eyre!("searchline out of index"));
    }
    let mut lines = lines
        .into_iter()
        .take((line + 1) as usize)
        .collect::<Vec<_>>();
    lines.reverse();
    println!("{:?}", lines);

    let mut my_level = usize::MAX;
    let mut res = vec![];
    for (idx, line_str) in lines.into_iter().enumerate() {
        if let Some((_, level, _)) = lazy_regex::regex_captures!("^[\\s>]*(#{1,})(.*)$", line_str) {
            println!("{} [{}]", level, my_level);
            let level = level.len();
            if my_level > level {
                my_level = level;
                res.push(line - idx as u64);
            }
            if !include_parents {
                break;
            } else if my_level <= 1 {
                break;
            }
        }
    }

    return Ok(res);
}
fn get_related_commits(config: &Config, time: DateTime<Utc>) -> color_eyre::Result<Vec<String>> {
    let transcription_config = config
        .transcription
        .clone()
        .ok_or_eyre("Expected transcription config to be loaded")?;

    let _ = git::check_out_create_branch(&transcription_config.git_target_branch, config)?;

    let res = git::git_command_wrapper(
        &["log", "--pretty='format:%H %ct'"],
        &config.git_directory,
        &config,
    )?;
    git::wrap_git_command_error(&res)?;

    if res.std_out.starts_with("fatal:") {
        if lazy_regex::regex_is_match!(
            "fatal: your current branch '[\\w]*' does not have any commits yet",
            &res.std_out
        ) {
            log::warn!(
                "Git branch '{}' has no commits; skipping",
                transcription_config.git_target_branch
            );
            return Ok(vec![]);
        } else {
            return Err(eyre!("Git returned error: '{}'", res.std_out));
        }
    }

    let res: Vec<(String, DateTime<Utc>)> = res
        .std_out
        .split("\n")
        .filter_map(|x| {
            let line = x.split(" ").collect::<Vec<_>>();
            if line.len() != 2 {
                return None;
            }
            Some((line[0], line[1]))
        })
        .map(|(commit_id, timestamp)| {
            (
                commit_id.to_owned(),
                DateTime::from_timestamp(
                    timestamp
                        .parse()
                        .expect("Expected timestamp to be only numbers"),
                    0,
                )
                .unwrap_or_default(),
            )
        })
        .collect();

    let cutoff_time = time.sub(transcription_config.time_window);
    let res = res;
    let res = res
        .into_iter()
        .filter(|(_, b)| *b >= cutoff_time)
        .map(|x| x.0)
        .collect::<Vec<_>>();
    Ok(res)
}

#[test]
fn test_get_changed_lines() {
    let patch = "diff --git a/abc.txt b/abc.txt
index a08dfdf..920c441 100644
--- a/abc.txt
+++ b/abc.txt
@@ -1,11 +1,11 @@
 a
 c
-c
+a
 e
 f
-f2
+22
 f3
-f4
+34
 g
 h
 h2
";
    let patch = patch::Patch::from_multiple(&patch).unwrap();
    let res = vec![
        (PathBuf::from_str("abc.txt").unwrap(), 2),
        (PathBuf::from_str("abc.txt").unwrap(), 5),
        (PathBuf::from_str("abc.txt").unwrap(), 7),
    ];

    assert_eq!(get_changed_lines(&patch).unwrap(), res);
}

#[test]
fn test_markdown_heading_parser() {
    let input = "content
# 1.0 Heading
content
    ## 1.2 Heading
content
## 1.3 Heading
content
>
>   # Heading
>   content
>   ## second heading
>   content
";
    let res_pattern = [
        (0, false, vec![]),
        (1, true, vec![1]),
        (4, false, vec![3]),
        (4, true, vec![3, 1]),
        (8, true, vec![8]),
        (11, true, vec![10, 8]),
    ];
    for (line, parent, res) in res_pattern {
        assert_eq!(
            get_related_markdown_headings(line, &input, parent).unwrap(),
            res,
            "Parsing line {}",
            line
        );
    }
}
