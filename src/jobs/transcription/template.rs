use color_eyre::eyre::{eyre, OptionExt};
use itertools::Itertools;

use super::{deepgram::TranscriptionResult, link::Link};

pub(crate) fn get_transcription_file(
    transcription: &TranscriptionResult,
    link: &Link,
) -> color_eyre::Result<String> {
    Ok(format!(
        "\
# Transcript '{}'

> _Links
>
{}

## Summary
{}

## Transcript
{}
",
        link.last_modified.format("%d.%m.%Y %H:%M"),
        format_tags(&transcription, &link)?,
        transcription.summary,
        format_paragraphs(&transcription, &link)?
    ))
}
fn format_tags(res: &TranscriptionResult, link: &Link) -> color_eyre::Result<String> {
    let mut res = res
        .topics
        .clone()
        .into_iter()
        .map(|x| {
            format!(
                "> [{}](topic://{})",
                x.topic,
                url_escape::encode_component(&x.topic).to_string()
            )
        })
        .collect_vec();
    res.push(format!("> [Source File]({})", format_link(link, None)?));
    Ok(res.join("\n"))
}
fn format_link(link: &Link, offset: Option<f64>) -> color_eyre::Result<String> {
    let mut res = match &link.link_target {
        crate::jobs::transcription::link::LinkType::FileSytemLink(x) => {
            let mut link = url_escape::encode_path(
                x.to_str()
                    .ok_or_eyre(eyre!("Expected Link path to be parsable; got {:?}", x))?,
            )
            .to_string();
            if !link.starts_with("/") {
                link = format!("/{}", link);
            }
            if let Some(x) = offset {
                link = format!("{}?time={}", link, x)
            }
            link
        }
        crate::jobs::transcription::link::LinkType::WebLink(link) => link.to_string(),
        crate::jobs::transcription::link::LinkType::OneDriveLink(onedrive) => {
            let mut link = onedrive
                .to_str()
                .ok_or_eyre(eyre!(
                    "Expected Link path to be parsable; got {:?}",
                    onedrive
                ))?
                .to_owned();
            if !link.starts_with("/") {
                link = format!("/{}", link);
            }
            format!("onedrive:{}", url_escape::encode_path(&link).to_string())
        }
    };
    if let Some(x) = offset {
        res = format!("transcript:({}):{}", x, res)
    }
    Ok(res)
}
fn format_paragraphs(res: &TranscriptionResult, link: &Link) -> color_eyre::Result<String> {
    let speakers = res.paragraphs.iter().map(|x| x.speaker).dedup().count();
    let mut speaker_colors = Vec::new();
    for _ in 0..speakers {
        let color = generate_random_color();
        let color = format!("#{:x}{:x}{:x}", color.0, color.1, color.2);
        speaker_colors.push(color);
    }
    let mut result = String::new();
    for x in &res.paragraphs {
        let speaker = x
            .speaker
            .ok_or_eyre(format!("Expected speaker to be set, got {:?}", x))?;
        result.push_str(&format!(
            "<mark style=\"background-color:{}\">[**Person {:2>0}**]({})</mark>: {}<br/>\n",
            speaker_colors[speaker],
            speaker,
            format_link(link, Some(x.start))?,
            x.sentences.iter().map(|x| x.clone().text).join(" ")
        ));
    }
    Ok(result)
}

// from https://en.wikipedia.org/wiki/HSL_and_HSV#HSV_to_RGB
// https://martin.ankerl.com/2009/12/09/how-to-create-random-colors-programmatically/
fn hsv_to_rgb(h: f64, s: f64, v: f64) -> (u8, u8, u8) {
    let h_i = (h * 6.0).floor() as i32;
    let f = h * 6.0 - h_i as f64;

    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);

    let (r, g, b) = match h_i {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        5 => (v, p, q),
        _ => panic!("Invalid hue value"),
    };

    (
        (r * 255.0).round() as u8,
        (g * 255.0).round() as u8,
        (b * 255.0).round() as u8,
    )
}

// https://martin.ankerl.com/2009/12/09/how-to-create-random-colors-programmatically/
fn generate_random_color() -> (u8, u8, u8) {
    let phi = (1. + (5_f64).sqrt()) / 2. - 1.; // Golden Ratio
    let rand = rand::random::<f64>(); // [0, 1)
    let h = (rand + phi) % 1.;
    hsv_to_rgb(h, 0.5, 0.95)
}
