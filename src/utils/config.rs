use std::{path::PathBuf, str::FromStr};

use reqwest::{header::HeaderValue, Url};

#[derive(Debug, Clone)]
pub struct Config {
    pub credentials: CredentialConfig,
    pub audio_sync: Option<AudioSyncConfig>,
}
impl Config {
    pub fn from_environment(audio_sync: bool) -> color_eyre::Result<Config> {
        Ok(Config {
            credentials: CredentialConfig::from_environment()?,
            audio_sync: if audio_sync {
                Some(AudioSyncConfig::from_environment()?)
            } else {
                None
            },
        })
    }
}

#[derive(Debug, Clone)]
pub struct CredentialConfig {
    pub onedrive_access_token_authorization: HeaderValue,
    pub onedrive_access_token_url: Url,
}

impl CredentialConfig {
    pub fn from_environment() -> color_eyre::Result<CredentialConfig> {
        Ok(CredentialConfig {
            onedrive_access_token_url: Url::parse(&std::env::var("ONEDRIVE_ACCESS_TOKEN_URL")?)?,
            onedrive_access_token_authorization: HeaderValue::from_str(&std::env::var(
                "ONEDRIVE_ACCESS_TOKEN_AUTHORIZATION",
            )?)?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct AudioSyncConfig {
    // OneNote
    pub onedrive_source_folder: String,

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
            onedrive_source_folder: std::env::var("ONEDRIVE_SOURCE_FOLDER")?,

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
