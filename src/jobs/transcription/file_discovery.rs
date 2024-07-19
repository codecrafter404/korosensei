use std::path::PathBuf;

use color_eyre::eyre::OptionExt as _;
use itertools::Itertools;

use crate::utils::{config::Config, git};

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
