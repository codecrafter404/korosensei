use std::{path::PathBuf, str::FromStr};

use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use color_eyre::eyre::{eyre, Context, OptionExt};
use env_logger::fmt::Timestamp;
use graph_rs_sdk::GraphClient;
use itertools::Itertools;
use reqwest::Url;
use serde_json::from_str;

#[derive(Debug, Clone, PartialEq, Eq)]
/// WARNING: this link struct is only applicable to links in the working directory
pub struct Link {
    pub link_target: LinkType,
    pub last_modified: DateTime<Utc>,
}
impl Link {
    /// WARNING: only accepts ABSOLUTE paths
    pub fn from_path(path: &PathBuf) -> color_eyre::Result<Link> {
        debug_assert!(path.is_absolute(), "Cant read from relative path");
        let content = std::fs::read_to_string(path)?;
        let mut link = Link::parse_link_file(&content)?;

        match link.link_target {
            LinkType::OneNoteLink(_) => {}
            _ => {}
        }

        link.validate_link()?;

        return Ok(link);
    }
    fn validate_link(&self) -> color_eyre::Result<()> {
        unimplemented!()
    }
    fn parse_link_file(content: &str) -> color_eyre::Result<Link> {
        let lines = content.split("\n").collect_vec();
        if lines.len() == 0 {
            return Err(eyre!("Can't parse empty link file"))?;
        }
        let line = lines[0];

        return Ok(if line.starts_with("onenote:") {
            let (_, timestamp, path) =
                lazy_regex::regex_captures!("onenote:\\((\\d{1,})\\):(.*)", line).ok_or_eyre(
                    format!(
                        "Expected onenote link in format onenote:(timestamp):/path; got {}",
                        line
                    ),
                )?;

            let timestamp = timestamp.parse::<i64>()?;
            let link = Link {
                link_target: LinkType::OneNoteLink(
                    PathBuf::from_str(path).wrap_err(format!("Expected path, got {}", path))?,
                ),
                last_modified: DateTime::from_timestamp(timestamp, 0)
                    .ok_or_eyre(format!("Failed to parse timestamp {}", timestamp))?,
            };
            link
        } else if ["http", "https"]
            .into_iter()
            .find(|x| line.starts_with(x))
            .is_some()
        {
            Link {
                link_target: LinkType::WebLink(
                    Url::parse(line).wrap_err(format!("Failed to parse URL: {}", line))?,
                ),
                last_modified: crate::utils::time::get_uninitalized_timestamp(),
            }
        } else {
            // has to be local file
            let path = PathBuf::from_str(line)
                .wrap_err(eyre!("Failed to parse file_system path: {}", line))?;
            Link {
                link_target: LinkType::FileSytemLink(path),
                last_modified: crate::utils::time::get_uninitalized_timestamp(),
            }
        });
    }
}

#[test]
fn test_link_parse() {
    let tests = vec![
        (
            "onenote:(1436809466):/assets/audio/audio1.mp3",
            Link {
                link_target: LinkType::OneNoteLink(
                    PathBuf::from_str("/assets/audio/audio1.mp3").unwrap(),
                ),
                last_modified: DateTime::from_timestamp(1436809466, 0).unwrap(),
            },
        ),
        (
            "C:\\User\\Koro-Sensei\\Music\\Savage Youth Theory.mp3",
            Link {
                link_target: LinkType::FileSytemLink(
                    PathBuf::from_str("C:\\User\\Koro-Sensei\\Music\\Savage Youth Theory.mp3")
                        .unwrap(),
                ),
                last_modified: crate::utils::time::get_uninitalized_timestamp(),
            },
        ),
        (
            "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
            Link {
                link_target: LinkType::WebLink(
                    Url::parse("https://www.youtube.com/watch?v=dQw4w9WgXcQ").unwrap(),
                ),
                last_modified: crate::utils::time::get_uninitalized_timestamp(),
            },
        ),
    ];
    for (input, output) in tests {
        assert_eq!(
            Link::parse_link_file(input).unwrap(),
            output,
            "input {}; expected {:?}",
            input,
            output
        );
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkType {
    /// Local link to path on the filesystem
    FileSytemLink(PathBuf),
    /// Link to a file in the web
    WebLink(Url),
    /// Link to a file hosted on onenote;
    OneNoteLink(PathBuf),
}
