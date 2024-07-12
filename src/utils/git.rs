use std::{path::Path, process::ExitStatus};

use color_eyre::eyre::eyre;

use super::config::Config;

#[derive(Debug, Clone)]
pub struct GitCommandOutput {
    pub status: ExitStatus,
    pub std_out: String,
    pub std_err: String,
    pub args: Vec<String>,
}
pub fn git_command_wrapper(
    args: &[&str],
    path: &Path,
    config: &Config,
) -> color_eyre::Result<GitCommandOutput> {
    let res = std::process::Command::new(config.git_exec.clone())
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

pub fn get_branches(config: &Config) -> color_eyre::Result<Vec<String>> {
    let res = git_command_wrapper(
        &["branch", "--list", "--no-color"],
        &config.git_directory,
        &config,
    )?;
    wrap_git_command_error(&res)?;
    let branches: Vec<_> = res
        .std_out
        .split("\n")
        .filter(|x| !x.is_empty())
        .map(|x| x[2..].to_owned())
        .collect(); // remove the 2 colums displaying the current status
    Ok(branches)
}

/// return bool -> indicates whether or not a branch has been created or not
pub fn check_out_create_branch(branch: &str, config: &Config) -> color_eyre::Result<bool> {
    let branches = get_branches(&config)?;
    if !branches.contains(&branch.to_owned()) {
        log::info!("Creating empty branch {}", branch);
        let res = git_command_wrapper(
            &["switch", "--orphan", branch],
            &config.git_directory,
            &config,
        )?;
        wrap_git_command_error(&res)?;
        return Ok(true);
    } else {
        let res = git_command_wrapper(&["checkout", branch], &config.git_directory, &config)?;
        wrap_git_command_error(&res)?;
        return Ok(false);
    }
}
