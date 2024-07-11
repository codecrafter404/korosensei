use clap::Parser;

mod jobs;
mod utils;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    env_logger::init();
    color_eyre::install()?;
    match dotenv::dotenv() {
        Ok(x) => {
            log::info!("Using .env: {:?}", x);
        }
        Err(x) => {
            log::info!("the .env file will be skipped: {}", x);
        }
    }

    let args = crate::utils::commandline::Args::parse();

    let config = crate::utils::config::Config::from_environment(args.audio_linker)?;

    if args.audio_linker {
        crate::jobs::audio_linker::link_audio(&config).await?;
    }
    return Ok(());
}
