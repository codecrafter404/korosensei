use std::ops::{Deref, Sub as _};
use std::path::PathBuf;
use std::str::FromStr;

use chrono::{Date, DateTime, Utc};
use color_eyre::eyre::{eyre, Context, OptionExt};

use crate::utils::config::Config;
use crate::utils::git::{self, git_command_wrapper};

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

    unimplemented!();
}

fn diff_commit(commit_id: &str, config: &Config) -> color_eyre::Result<Vec<(PathBuf, i32)>> {
    let res = git::git_command_wrapper(
        &["diff", "-p", &format!("{}~1", commit_id), commit_id],
        &config.git_directory,
        config,
    )?;
    git::wrap_git_command_error(&res)?;
    let patches = patch::Patch::from_multiple(&res.std_out);
    if let Ok(patches) = patches {
        let lines = get_changed_lines(&patches)?;
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

/// return: those paths are only relative
fn get_commit_files(config: &Config, commit_id: &str) -> color_eyre::Result<Vec<PathBuf>> {
    //TODO:  git diff-tree --no-commit-id --name-only bcabfc59b2faec296d3b2804945db1cbf8665629 -r

    let res = git::git_command_wrapper(
        &[
            "diff-tree",
            "--no-commit-id",
            "--name-only",
            commit_id,
            "-r",
        ],
        &config.git_directory,
        config,
    )?;
    git::wrap_git_command_error(&res)?;

    let res = res
        .std_out
        .split("\n")
        .map(|x| PathBuf::from_str(x))
        .collect::<Result<Vec<_>, _>>()
        .wrap_err("Expected to receive valid paths")?;
    Ok(res)
}
fn get_related_commits(config: &Config, time: DateTime<Utc>) -> color_eyre::Result<Vec<String>> {
    let transcription_config = config
        .transcription
        .clone()
        .ok_or_eyre("Expected transcription config to be loaded")?;

    let _ = git::check_out_create_branch(&transcription_config.git_target_branch, config)?;

    //TODO: git log --pretty="format:%H %ct"
    //example output:
    //  <commit-hash>                           <UNIX-TIME>
    // b0edc6539e77ad73bdc26f1297137ec8ce33b808 1720786892
    // a1b06e8eca7ede4080e01e0ce20de85a7b70d5cf 1720784184
    // 602c07096d316622c40e001fbd00a9647fd8d4f3 1720737103
    // 09c61c03e78c5e8e1344f97d35c74018fe83507d 1720733077
    // 708033b856f6f4795cec84be33258896a78ac3a8 1720728602
    // cd77c2c3a92709ff0c1b608e02582a7bbbb3a6e9 1720723871
    // 2ed1f8f59afaeb70d42e8f7b8da82336c15ff19d 1720720206
    // b0ce59e83500f147825d2788f9f6593afa73cbcf 1720712972
    // 39ef6b0cf48365985eb8c9b308dcfe441d566430 1720628188
    // 3a7763c5da286ae7fce37fde71797cba5f39cf0a 1720623611
    // 887fdc6369428a794e8eb7875df85450faa8f882 1720608820

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
    let patch = "diff --git a/abc.txt b/abc.txt\
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
        (PathBuf::from_str("/abc.txt").unwrap(), 2),
        (PathBuf::from_str("/abc.txt").unwrap(), 5),
        (PathBuf::from_str("/abc.txt").unwrap(), 7),
    ];

    assert_eq!(get_changed_lines(&patch).unwrap(), res);
}
