use std::{intrinsics::sqrtf64, path::PathBuf};

use crate::utils::config::Config;

use super::{deepgram::TranscriptionResult, link::Link};

pub(crate) fn get_transcription_file(transcription: &TranscriptionResult, link: &Link) -> String {
    format!(
        "\
# Transcript '{}'

> _Links
>
> [Source File]({})


",
        link.last_modified.format("%d/%m/%Y %H:%M"),
        format_link(&link)
    )
}
fn format_link(link: &Link) -> String {
    unimplemented!()
}

pub(crate) fn get_link_node(
    transcript: &PathBuf,
    link: &Link,
    conf: &Config,
) -> color_eyre::Result<()> {
    unimplemented!()
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
