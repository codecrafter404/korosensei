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
{}
> [Source File]({})

## Summary
{}

## Transcript
{}
",
        link.last_modified.format("%d/%m/%Y %H:%M"),
        format_tags(&transcription),
        format_link(&link, None)?,
        transcription.summary,
        format_paragraphs(&transcription, &link)?
    ))
}
fn format_tags(res: &TranscriptionResult) -> String {
    res.topics
        .clone()
        .into_iter()
        .map(|x| format!("> [{}](topic://{})", x.topic, x.topic))
        .join("\n")
}
fn format_link(link: &Link, offset: Option<f64>) -> color_eyre::Result<String> {
    let mut res = match &link.link_target {
        crate::jobs::transcription::link::LinkType::FileSytemLink(x) => {
            let mut link = x
                .to_str()
                .ok_or_eyre(eyre!("Expected Link path to be parsable; got {:?}", x))?
                .to_owned();
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
            format!("onedrive:{}", link)
        }
    };
    if let Some(x) = offset {
        res = format!("transcript:({}):{}", x, res)
    }
    Ok(res)
}
fn format_paragraphs(res: &TranscriptionResult, link: &Link) -> color_eyre::Result<String> {
    let color = generate_random_color();
    let color = format!(
        "#{:x}{:x}{:x}",
        color.0.floor() as i8,
        color.1.floor() as i8,
        color.2.floor() as i8
    );
    let mut result = String::new();
    for x in &res.paragraphs {
        format!(
            "<mark style=\"background-color:{}\">[**Person {:2>0}**]({})</mark>: {}",
            color,
            x.speaker
                .ok_or_eyre(format!("Expected speaker to be set, got {:?}", x))? as i32,
            format_link(link, Some(x.start))?,
            x.sentences.iter().map(|x| x.clone().text).join(" ")
        );
        result = format!("{}\n", result);
    }
    Ok(result)
}

// from https://en.wikipedia.org/wiki/HSL_and_HSV#HSV_to_RGB
fn hsv_to_rgb(h: f64, s: f64, v: f64) -> (f64, f64, f64) {
    assert!(0. <= h && h < 360.);
    assert!(0. <= s && s <= 1.);
    assert!(0. <= v && v <= 1.);
    let c = v * s;
    let h1 = h / 60.;
    let x = c * (1. - (h1 % 2. - 1.).abs());
    let (r1, g1, b1) = if 0. <= h1 && h1 < 1. {
        (c, x, 0.)
    } else if 1. <= h1 && h1 < 2. {
        (x, c, 0.)
    } else if 2. <= h1 && h1 < 3. {
        (0., c, x)
    } else if 3. <= h1 && h1 < 4. {
        (0., x, c)
    } else if 4. <= h1 && h1 < 5. {
        (x, 0., c)
    } else if 5. <= h1 && h1 < 6. {
        (c, 0., x)
    } else {
        (0., 0., 0.)
    };

    let m = v - c;
    (r1 + m, g1 + m, b1 + m)
}

// https://martin.ankerl.com/2009/12/09/how-to-create-random-colors-programmatically/
fn generate_random_color() -> (f64, f64, f64) {
    let phi = (1. + (5_f64).sqrt()) / 2.; // Golden Ratio
    let rand = rand::random::<f64>(); // [0, 1)
    let h = (rand + phi) % 1.;
    hsv_to_rgb(h, 0.5, 0.95)
}
