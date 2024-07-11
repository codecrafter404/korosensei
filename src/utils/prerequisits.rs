use color_eyre::eyre::Context;

pub fn check_prerequisits() -> color_eyre::Result<()> {
    let _ = which::which("git").wrap_err("Git is expected to be installed and in your $PATH")?;
    unimplemented!();
}
