use std::{path::PathBuf, str::FromStr};

use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use graph_rs_sdk::GraphClient;
use reqwest::Url;

#[derive(Debug, Clone, PartialEq, Eq)]
/// WARNING: this link struct is only applicable to links in the working directory
pub struct Link {
    pub link_target: LinkType,
    pub last_modified: DateTime<Utc>,
}
impl Link {
    pub fn from_path(path: &PathBuf) -> color_eyre::Result<Link> {
        unimplemented!()
    }
    fn validate_link(link: &Link) -> color_eyre::Result<()> {
        unimplemented!()
    }
    fn parse_link_file(content: &str) -> color_eyre::Result<Link> {
        unimplemented!()
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
                last_modified: NaiveDate::from_ymd_opt(1, 1, 1)
                    .unwrap()
                    .and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
                    .and_local_timezone(Utc)
                    .unwrap(),
            },
        ),
        (
            "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
            Link {
                link_target: LinkType::WebLink(Url::parse("").unwrap()),
                last_modified: NaiveDate::from_ymd_opt(1, 1, 1)
                    .unwrap()
                    .and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
                    .and_local_timezone(Utc)
                    .unwrap(),
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
