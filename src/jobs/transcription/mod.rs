use std::path::{Path, PathBuf};

use crate::utils::git;
use chrono::{DateTime, TimeZone, Utc};
use color_eyre::eyre::OptionExt;

use crate::utils::config::Config;

mod deepgram;
mod file_discovery;
mod file_meta;
mod link;
mod markdown;
mod template;

pub async fn transcribe_audio(conf: &Config) -> color_eyre::Result<()> {
    let files_to_transcribe = file_discovery::discover_files(conf)?;

    //TODO: determine which files have to be transcripted -> transcribe them

    //TODO: link the transcriptions to corresponding notes
    unimplemented!();
}
/// expects to be in the right git branch
async fn process_file(
    conf: &Config,
    file_to_transcribe: PathBuf,
    deepgram: ::deepgram::Deepgram,
    graph: &graph_rs_sdk::GraphClient,
) -> color_eyre::Result<()> {
    let link = link::Link::from_path(&file_to_transcribe, &conf)?;
    let transcription_result = deepgram::transcribe_link(&link, conf, deepgram, graph).await?;
    let file_content = template::get_transcription_file(&transcription_result, &link)?;
    let target_file_name = format!(
        "{}.transcript.md",
        file_to_transcribe
            .file_name()
            .ok_or_eyre("expected to get filename")?
            .to_str()
            .ok_or_eyre("Expected to parse filename")?
    );
    let transcription_config = conf
        .transcription
        .clone()
        .ok_or_eyre("Expected transcription config to be set")?;
    let dir = conf.git_directory.join(
        transcription_config
            .transcription_script_search_path
            .strip_prefix("/")
            .unwrap_or(&transcription_config.transcription_script_search_path),
    );
    std::fs::create_dir_all(dir.clone())?;
    let path = dir.join(target_file_name);
    std::fs::write(path.clone(), file_content)?;

    let correlating_files = markdown::discorver_correlating_files(link.last_modified, conf).await?;

    //TODO: handle correlating files

    Ok(())
}
