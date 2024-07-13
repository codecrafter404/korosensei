use std::path::PathBuf;

use crate::utils::config::Config;

pub(crate) fn get_transcription_file(
    summmary: &str,
    transcription: Vec<String>,
    conf: &Config,
) -> color_eyre::Result<String> {
    unimplemented!()
}

pub(crate) fn get_link_node(transcript: &PathBuf, conf: &Config) -> color_eyre::Result<()> {
    unimplemented!()
}
