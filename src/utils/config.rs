use std::{path::PathBuf, str::FromStr};

use color_eyre::eyre::Context;
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
            onedrive_access_token_url: Url::parse(
                &dotenv::var("ONEDRIVE_ACCESS_TOKEN_URL")
                    .wrap_err("Expected ONEDRIVE_ACCESS_TOKEN_URL to be set")?,
            )?,
            onedrive_access_token_authorization: HeaderValue::from_str(
                &dotenv::var("ONEDRIVE_ACCESS_TOKEN_AUTHORIZATION")
                    .wrap_err("Expected ONEDRIVE_ACCESS_TOKEN_AUTHORIZATION to be set")?,
            )?,
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
            onedrive_source_folder: dotenv::var("ONEDRIVE_SOURCE_FOLDER")
                .wrap_err("Expected ONEDRIVE_SOURCE_FOLDER to be set")?,

            git_directory: PathBuf::from_str(
                &dotenv::var("GIT_DIRECTORY").wrap_err("Expected GIT_DIRECTORY to be set")?,
            )?,

            git_branch: dotenv::var("GIT_BRANCH").wrap_err("Expected GIT_BRANCH to be set")?,

            git_destination_folder: PathBuf::from_str(
                &dotenv::var("GIT_DESTINATION_FOLDER")
                    .wrap_err("Expected GIT_DESTINATION_FOLDER to be set")?,
            )?,

            permitted_file_types: dotenv::var("PERMITTED_FILE_TYPES")
                .wrap_err("Expected PERMITTED_FILE_TYPES to be set")?
                .replace(" ", "")
                .split(",")
                .map(|x| x.strip_prefix(".").unwrap_or(x).to_owned())
                .collect(),
        });
    }
}
