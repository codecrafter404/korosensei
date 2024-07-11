use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Args {
    #[arg(short, long)]
    pub audio_linker: bool,
}
