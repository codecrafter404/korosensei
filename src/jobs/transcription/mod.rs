use std::path::{Path, PathBuf};

use crate::utils::git;
use chrono::{DateTime, TimeZone, Utc};
use color_eyre::eyre::OptionExt;

use crate::utils::config::Config;

mod file_discovery;
mod file_meta;

pub async fn transcribe_audio(conf: &Config) -> color_eyre::Result<()> {
    //TODO: determine which files have to be transcripted -> transcribe them

    //TODO: link the transcriptions to corresponding notes
    unimplemented!();
}
