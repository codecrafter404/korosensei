use color_eyre::eyre::Context;

pub fn check_prerequisits(conf: &mut super::config::Config) -> color_eyre::Result<()> {
    let git_path =
        which::which("git").wrap_err("Git is expected to be installed and in your $PATH")?;
    conf.git_directory = git_path;

    unimplemented!();
}
