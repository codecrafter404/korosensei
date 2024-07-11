use std::{path::Path, process::ExitStatus};

use color_eyre::eyre::eyre;

#[derive(Debug, Clone)]
pub struct GitCommandOutput {
    pub status: ExitStatus,
    pub std_out: String,
    pub std_err: String,
    pub args: Vec<String>,
}
pub fn git_command_wrapper(args: &[&str], path: &Path) -> color_eyre::Result<GitCommandOutput> {
    let res = std::process::Command::new("git")
        .current_dir(path)
        .args(args)
        .output()?;

    Ok(GitCommandOutput {
        status: res.status,
        std_out: String::from_utf8(res.stdout)?,
        std_err: String::from_utf8(res.stderr)?,
        args: args.into_iter().map(|x| x.to_owned().to_owned()).collect(),
    })
}

pub fn wrap_git_command_error(res: &GitCommandOutput) -> color_eyre::Result<()> {
    if !res.status.success() {
        return Err(eyre!(
            "Git command({:?}) failed: {:?} ({:?})",
            res.args,
            res,
            res.status.code()
        ));
    }
    Ok(())
}

pub const GIT_AUTHOR: &str = "Koro-sensei <koro-sensei@ansatsu-anime.com>";
