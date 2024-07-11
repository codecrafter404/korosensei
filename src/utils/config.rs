use std::{path::PathBuf, str::FromStr};

use graph_rs_sdk::header::HeaderValue;
use reqwest::Url;

struct Config {}

#[derive(Debug, Clone)]
pub struct AudioSyncConfig {
    // OneNote
    pub onedrive_access_token_url: Url,
    pub onedrive_access_token_authorization: HeaderValue,
    pub onedrive_source_folder: PathBuf,

    // Git
    pub git_directory: PathBuf,
    pub git_branch: String,
    pub git_destination_folder: PathBuf,

    // General Settings
    pub permitted_file_types: Vec<String>,
}

impl AudioSyncConfig {
    pub fn from_environment() -> color_eyre::Result<AudioSyncConfig> {
        return Ok(AudioSyncConfig {
            onedrive_access_token_url: Url::parse(&std::env::var("ONEDRIVE_ACCESS_TOKEN_URL")?)?,
            onedrive_access_token_authorization: HeaderValue::from_str(&std::env::var(
                "ONEDRIVE_ACCESS_TOKEN_AUTHORIZATION",
            )?)?,
            onedrive_source_folder: PathBuf::from_str(&std::env::var("ONEDRIVE_SOURCE_FOLDER")?)?,

            git_directory: PathBuf::from_str(&std::env::var("GIT_DIRECTORY")?)?,
            git_branch: std::env::var("GIT_BRANCH")?,
            git_destination_folder: PathBuf::from_str(&std::env::var("GIT_DESTINATION_FOLDER")?)?,

            permitted_file_types: std::env::var("PERMITTED_FILE_TYPES")?
                .split(",")
                .map(|x| x.strip_prefix(".").unwrap_or(x).to_owned())
                .collect(),
        });
    }
}
