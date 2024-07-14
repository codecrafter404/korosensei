use std::{path::PathBuf, str::FromStr};

use chrono::Duration;
use color_eyre::eyre::Context;
use reqwest::{header::HeaderValue, Url};

use super::credentials::OneDriveCredentialsResponse;

#[derive(Debug, Clone)]
pub struct Config {
    pub credentials: CredentialConfig,
    pub audio_sync: Option<AudioSyncConfig>,
    pub transcription: Option<TranscriptionConfig>,
    pub git_directory: PathBuf,
    pub timezone: chrono_tz::Tz,
    pub git_exec: PathBuf,
}
impl Config {
    pub fn from_environment(audio_sync: bool, transcription: bool) -> color_eyre::Result<Config> {
        Ok(Config {
            credentials: CredentialConfig::from_environment()?,
            audio_sync: if audio_sync {
                Some(AudioSyncConfig::from_environment()?)
            } else {
                None
            },
            git_directory: PathBuf::from_str(
                &dotenv::var("GIT_DIRECTORY").wrap_err("Expected GIT_DIRECTORY to be set")?,
            )?,
            timezone: dotenv::var("TIMEZONE")
                .wrap_err("Expected TIMEZONE to be set")?
                .parse()?,
            transcription: if transcription {
                Some(TranscriptionConfig::from_environment()?)
            } else {
                None
            },
            git_exec: PathBuf::from_str(&dotenv::var("GITPATH").unwrap_or("".to_owned()))
                .unwrap_or(PathBuf::new()),
        })
    }
}

#[derive(Debug, Clone)]
pub struct TranscriptionConfig {
    pub transcription_script_search_path: PathBuf,
    pub git_source_branch: String,
    pub git_target_branch: String,
    pub git_source_path: PathBuf,
    pub time_window: Duration, // past n minutes
    pub include_parent: bool,
    pub deepgram_key: String,
}
impl TranscriptionConfig {
    pub fn from_environment() -> color_eyre::Result<TranscriptionConfig> {
        Ok(TranscriptionConfig {
            transcription_script_search_path: PathBuf::from_str(
                &dotenv::var("TRANSCRIPTION_SCRIPT_SEARCH_PATH")
                    .wrap_err("Expected TRANSCRIPTION_SCRIPT_SEARCH_PATH to be set")?,
            )?,
            git_source_branch: dotenv::var("TRANSCRIPTION_AUDIO_BRANCH")
                .wrap_err("Expected TRANSCRIPTION_AUDIO_BRANCH to be set")?,
            git_target_branch: dotenv::var("TRANSCRIPTION_GIT_BRANCH")
                .wrap_err("Expected TRANSCRIPTION_GIT_BRANCH to be set")?,
            git_source_path: PathBuf::from_str(
                &dotenv::var("TRANSCRIPTION_AUDIO_SOURCE_DIR")
                    .wrap_err("Expected TRANSCRIPTION_AUDIO_SOURCE_DIR to be set")?,
            )?,
            time_window: chrono::Duration::minutes(
                dotenv::var("TRANSCRIPTION_TIME_WINDOW")
                    .unwrap_or("100".to_owned())
                    .parse::<i64>()
                    .wrap_err("Failed to parse TRANSCRIPTION_TIME_WINDOW")?,
            ),
            include_parent: vec!["y".to_owned(), "yes".to_owned(), "1".to_owned()].contains(
                &dotenv::var("TRANSCRIPTION_AUDIO_SOURCE_DIR").unwrap_or("no".to_owned()),
            ),
            deepgram_key: dotenv::var("TRANSCRIPTION_DEEPGRAM_KEY")
                .wrap_err("Expected TRANSCRIPTION_DEEPGRAM_KEY to be set")?,
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
    // OneDrive
    pub onedrive_source_folder: String,

    // Git
    pub git_branch: String,
    pub git_destination_folder: PathBuf,

    // General Settings
    pub permitted_file_types: Vec<String>,
}

impl AudioSyncConfig {
    pub fn from_environment() -> color_eyre::Result<AudioSyncConfig> {
        return Ok(AudioSyncConfig {
            onedrive_source_folder: dotenv::var("ONEDRIVE_SOURCE_DIR")
                .wrap_err("Expected ONEDRIVE_SOURCE_DIR to be set")?,

            git_branch: dotenv::var("AUDIO_GIT_BRANCH")
                .wrap_err("Expected AUDIO_GIT_BRANCH to be set")?,

            git_destination_folder: PathBuf::from_str(
                &dotenv::var("AUDIO_TARGET_DIR").wrap_err("Expected AUDIO_TARGET_DIR to be set")?,
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
