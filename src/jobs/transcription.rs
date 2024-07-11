use std::path::Path;

use crate::utils::config::Config;

pub async fn transcribe_audio(conf: &Config) -> color_eyre::Result<()> {
    //TODO: determine which files have to be transcripted -> transcribe them

    //TODO: link the transcriptions to corresponding notes
    unimplemented!();
}

// first try through the filename
// second try through git blame
fn extract_file_change_date(file: &Path, conf: &Config) -> color_eyre::Result<()> {
    let name = file
        .file_name()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default();
    if let Some((_, dd, mm, yyyy, hh, mi)) = lazy_regex::regex_captures!(
        "^\\D*(\\d{1,2})[\\.\\-'](\\d{1,2})[\\.\\-'](\\d{1,4})\\D*(\\d{1,2})[\\.\\-'](\\d{1,2}).*$",
        name
    ) {
        let dd = format!("{:2>0}", dd);
    }
    unimplemented!()
}
