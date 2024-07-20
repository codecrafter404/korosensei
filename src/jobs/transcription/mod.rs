use std::path::{Path, PathBuf};

use crate::utils::git;
use chrono::{DateTime, Duration, TimeZone, Utc};
use color_eyre::eyre::{eyre, Context, OptionExt};
use itertools::Itertools;
use link::Link;
use markdown::CorrelatingFile;

use crate::utils::config::Config;

mod deepgram;
mod file_discovery;
mod file_meta;
mod link;
pub mod markdown;
mod template;

pub async fn transcribe_audio(conf: &Config) -> color_eyre::Result<()> {
    let transcription_conf = conf
        .transcription
        .clone()
        .ok_or_eyre("Expected transcription conf to be initialized")?;

    git::check_out_create_branch(&transcription_conf.git_source_branch, &conf)?;

    let files_to_transcribe = file_discovery::discover_files(conf)?;

    let deepgram = ::deepgram::Deepgram::new(transcription_conf.deepgram_key);
    let credentials =
        crate::utils::credentials::get_onedrive_credentials(&conf.credentials).await?;

    let graph = graph_rs_sdk::GraphClient::new(credentials.token);

    git::check_out_create_branch(&transcription_conf.git_source_branch, &conf)?;

    let mut links = Vec::new();
    for file in files_to_transcribe {
        match Link::from_path(&file, &conf) {
            Ok(x) => links.push((file, x)),
            Err(why) => {
                log::error!("Failed to parse link for file {:?}: {:?}", file, why);
            }
        }
    }

    if links.is_empty() {
        log::info!("didn't get any (new) links");
        return Ok(());
    }

    git::check_out_create_branch(&transcription_conf.git_target_branch, &conf)?;

    let (blamed_files, _) =
        git::blame::BlamedFile::blame_all(&conf).wrap_err("Failed to blame directory tree")?;
    println!("blamed_files: {:?}", blamed_files);
    let mut files_to_link = Vec::new();
    for (file, link) in links {
        match process_file(conf, file.clone(), link.clone(), &deepgram, &graph).await {
            Ok(x) => {
                files_to_link.push(x);
            }
            Err(why) => {
                log::error!("Failed to proccess link: {:?}", why);
            }
        }
    }

    let mut processed = Vec::new();
    // link transcripts to correlating files
    // TODO: make more efficient to not read all files multiple times
    for (link, transcript_path) in files_to_link {
        let cut_of_date = link.last_modified - transcription_conf.time_window;
        let correlating_files = blamed_files
            .clone()
            .into_iter()
            .map(|x| x.to_correlating_file(&conf, cut_of_date.clone()))
            .collect::<Result<Vec<_>, _>>();
        match handle_correlating_files(
            correlating_files,
            transcript_path.clone(),
            &link.last_modified,
        ) {
            Ok(_) => processed.push(transcript_path.clone()),
            Err(why) => {
                log::error!(
                    "Failed to link_correlating_files for transcript {:?}: {:?}",
                    transcript_path,
                    why
                );
            }
        }
    }

    // commit changes
    if processed.len() > 0 {
        git::wrap_git_command_error(&git::git_command_wrapper(
            &["add", "*"],
            &conf.git_directory,
            conf,
        )?)?;
        git::wrap_git_command_error(&git::git_command_wrapper(
            &[
                "commit",
                "-m",
                &format!(
                    "transcribed: {}",
                    processed
                        .iter()
                        .map(|x| x
                            .file_name()
                            .unwrap_or_default()
                            .to_str()
                            .unwrap_or_default())
                        .join(",")
                ),
                "--author",
                git::GIT_AUTHOR,
            ],
            &conf.git_directory,
            conf,
        )?)?;
    }
    Ok(())
}
/// expects to be in the right git branch
async fn process_file(
    conf: &Config,
    file_to_transcribe: PathBuf,
    link: Link,
    deepgram: &::deepgram::Deepgram,
    graph: &graph_rs_sdk::GraphClient,
) -> color_eyre::Result<(Link, PathBuf)> {
    let transcription_config = conf
        .transcription
        .clone()
        .ok_or_eyre("Expected transcription config to be set")?;
    log::info!("Proccessing link '{:?}'", file_to_transcribe);
    let transcription_result = deepgram::transcribe_link(&link, conf, deepgram, graph)
        .await
        .wrap_err(eyre!("Failed to transcribe file"))?;
    let file_content = template::get_transcription_file(&transcription_result, &link)?;
    let file_without_link_extension = file_to_transcribe
        .file_name()
        .ok_or_eyre("expected to get filename")?
        .to_str()
        .ok_or_eyre("Expected to parse filename")?
        .to_string()
        .strip_suffix(".link")
        .ok_or_eyre("Infallible")?
        .to_string();
    let target_file_name = format!("{}.transcript.md", file_without_link_extension);
    let dir = conf.git_directory.join(
        transcription_config
            .transcription_target_path
            .strip_prefix("/")
            .unwrap_or(&transcription_config.transcription_target_path),
    );

    std::fs::create_dir_all(dir.clone())?;
    let path = dir.join(target_file_name.clone());
    std::fs::write(path.clone(), file_content)?;

    Ok((link, path))
}
fn handle_correlating_files(
    files: color_eyre::Result<Vec<Option<CorrelatingFile>>>,
    transcript: PathBuf,
    time: &DateTime<Utc>,
) -> color_eyre::Result<()> {
    let files = files?;
    let files = files.into_iter().filter_map(|x| x).collect_vec();
    log::info!("Got {} files to link", files.len());

    for file in files {
        match file.link_to_transcript(transcript.clone(), &file.content, time) {
            Ok(x) => match std::fs::write(file.path.clone(), x) {
                Ok(_) => {
                    log::info!("Successfully linked {:?} -> {:?}", transcript, file.path);
                }
                Err(why) => {
                    log::error!(
                        "Failed to write updated file {:?} (while linking {:?}): {:?}",
                        file.path,
                        transcript,
                        why
                    );
                }
            },
            Err(why) => {
                log::error!(
                    "Failed to link transcript {:?} to file {:?}: {:?}",
                    transcript,
                    file.path,
                    why
                );
            }
        }
    }
    Ok(())
}
