use std::{
    path::{Path, PathBuf},
    str::FromStr as _,
};

use crate::utils::{
    config::{Config, CredentialConfig},
    git,
};
use chrono::{DateTime, TimeZone, Utc};
use color_eyre::eyre::OptionExt;
use reqwest::{header::HeaderValue, Url};
// first try through the filename
// second try through git
pub(crate) fn extract_file_change_date(
    file: &Path,
    conf: &Config,
) -> color_eyre::Result<chrono::DateTime<Utc>> {
    let name = file
        .file_name()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default();

    if let Some(res) = match_date_from_name(name, conf)? {
        return Ok(res);
    }

    Ok(match_date_from_git(file, conf)?)
}
fn match_date_from_git(file: &Path, conf: &Config) -> color_eyre::Result<chrono::DateTime<Utc>> {
    let transcription_config = conf
        .transcription
        .clone()
        .ok_or_eyre("Expected transcription to be configured")?;

    let _ =
        crate::utils::git::check_out_create_branch(&transcription_config.git_source_branch, &conf)?;

    let res = git::git_command_wrapper(
        &[
            "rev-list",
            "-1",
            "HEAD",
            file.to_str().ok_or_eyre("Expected file to be a a path")?,
        ],
        &conf.git_directory,
        &conf,
    )?;
    git::wrap_git_command_error(&res)?;

    let commit_id = res.std_out.trim();
    let res = git::git_command_wrapper(
        &["show", &commit_id, "--quiet", "--pretty=%ct"],
        &conf.git_directory,
        &conf,
    )?;
    git::wrap_git_command_error(&res)?;
    let date = chrono::DateTime::from_timestamp(res.std_out.trim().parse()?, 0)
        .ok_or_eyre("Expected the git commit date to be a unix timestamp")?;
    return Ok(date);
}

fn match_date_from_name(name: &str, conf: &Config) -> color_eyre::Result<Option<DateTime<Utc>>> {
    if let Some((_, dd, mm, yyyy, hh, mi)) = lazy_regex::regex_captures!(
        "^\\D*(\\d{1,2})[\\.\\-'](\\d{1,2})[\\.\\-'](\\d{1,4})\\D*(\\d{1,2})[\\.\\-'](\\d{1,2}).*$",
        name
    ) {
        let dd = dd.parse::<u32>()?;
        let mm = mm.parse::<u32>()?;
        let mut yyyy = yyyy.parse::<u32>()?;
        if yyyy < 1000 {
            yyyy += 2000 // should be enough for the next ~974 years :)
        }

        let hh = hh.parse::<u32>()?;
        let mi = mi.parse::<u32>()?;

        if (dd >= 1 && dd <= 31) && (mm >= 1 && mm <= 12) && (hh <= 24 && mi <= 60) {
            let date = conf
                .timezone
                .with_ymd_and_hms(yyyy as i32, mm, dd, hh, mi, 0);
            let date = date
                .single()
                .ok_or_eyre("Expected timezone to have an single unique result")?;
            let utc = date.with_timezone(&Utc);
            return Ok(Some(utc));
        }
    }
    Ok(None)
}

#[test]
fn test_date_from_name() {
    let res_list = [
        (
            "Recording 19.12.2020 04.20.mp3",
            Some(DateTime::from_timestamp(1608348000, 0).unwrap()),
        ),
        (
            "Recording 19-12-2020 04.20 adf.mp3",
            Some(DateTime::from_timestamp(1608348000, 0).unwrap()),
        ),
        (
            "Recording 19.12.20 04'20 hello.wav",
            Some(DateTime::from_timestamp(1608348000, 0).unwrap()),
        ),
        (
            "Recording 1.1.15 3'3.mp3",
            Some(DateTime::from_timestamp(1420077780, 0).unwrap()),
        ),
        ("Recording 1.69.42 3'3.mp3", None),
        ("Recording without date.mp3", None),
    ];

    let conf = Config {
        credentials: CredentialConfig {
            onedrive_access_token_authorization: HeaderValue::from_static(""),
            onedrive_access_token_url: Url::from_str("http://google.com/").unwrap(),
        },
        audio_sync: None,
        git_directory: PathBuf::new(),
        timezone: "Europe/Berlin".parse().unwrap(),
        git_exec: PathBuf::new(),
        transcription: None,
    };

    for (a, b) in res_list {
        assert_eq!(match_date_from_name(a, &conf).unwrap(), b);
    }
}
