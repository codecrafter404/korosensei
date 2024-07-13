use clap::Parser;

mod jobs;
mod utils;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    env_logger::init();
    color_eyre::install()?;
    let args = crate::utils::commandline::Args::parse();
    match dotenv::dotenv() {
        Ok(x) => {
            log::info!("Using .env: {:?}", x);
        }
        Err(x) => {
            log::info!("the .env file will be skipped: {}", x);
        }
    }
    let mut config =
        crate::utils::config::Config::from_environment(args.audio_linker, args.transcription)?;

    crate::utils::prerequisits::check_prerequisits(&mut config)?;

    if args.audio_linker {
        crate::jobs::audio_linker::link_audio(&config).await?;
    }
    if args.transcription {
        crate::jobs::transcription::transcribe_audio(&config).await?;
    }
    return Ok(());
}
