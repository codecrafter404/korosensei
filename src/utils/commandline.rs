use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about, arg_required_else_help = true)]
pub struct Args {
    #[arg(short, long)]
    pub audio_linker: bool,
    #[arg(short, long)]
    pub transcription: bool,
}
