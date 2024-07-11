use std::{path::Path, process::ExitStatus};

use color_eyre::eyre::{eyre, OptionExt as _};
use futures::StreamExt as _;
use graph_rs_sdk::{http::HttpResponseExt as _, GraphClient, ODataQuery as _};
use serde::Deserialize;

use crate::utils::config::Config;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OneDriveChildren {
    name: String,
    // file: Option<OneDriveFile>,
    folder: Option<OneDriveFolder>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OneDriveFolder {}
// #[derive(Debug, Deserialize)]
// #[serde(rename_all = "camelCase")]
// struct OneDriveFile {}
pub async fn link_audio(config: &Config) -> color_eyre::Result<()> {
    let credential_config = config.credentials.clone();
    let audio_sync = config
        .clone()
        .audio_sync
        .ok_or_eyre("Expected audio_sync config to be initialized")?;

    let github_repo_root = audio_sync.git_directory;

    //TODO: validate that the branch exists

    let res = git_command_wrapper(&["branch", "--list", "--no-color"], &github_repo_root)?;
    wrap_git_command_error(&res)?;
    let branches: Vec<_> = res.std_out.split("\n").map(|x| x[2..].to_owned()).collect(); // remove the 2 colums displaying the current status
    if !branches.contains(&audio_sync.git_branch) {
        log::info!("Creating empty branch {}", audio_sync.git_branch);
        let res = git_command_wrapper(
            &["switch", "--orphan", &audio_sync.git_branch],
            &github_repo_root,
        )?;
        wrap_git_command_error(&res)?
    } else {
        let res = git_command_wrapper(&["checkout", &audio_sync.git_branch], &github_repo_root)?;
        wrap_git_command_error(&res)?;
    }
    let git_target_path = github_repo_root.join(
        audio_sync
            .git_destination_folder
            .strip_prefix("/")
            .unwrap_or(&audio_sync.git_destination_folder),
    );
    if git_target_path.is_file() {
        return Err(eyre!(
            "Git source dir {:?} is a file",
            git_target_path.to_str()
        ));
    }
    if !git_target_path.exists() {
        std::fs::create_dir_all(&git_target_path)?;
    }

    let github_files: Result<Vec<_>, _> = std::fs::read_dir(&git_target_path)?
        .filter(|x| match x {
            Ok(x) => match x.file_type() {
                Ok(x) => x.is_file(),
                Err(_) => false,
            },
            Err(_) => false,
        })
        .map(|x| x.map(|x| x.path()))
        .collect();
    let github_files = github_files?;

    // OneDrive
    let token = crate::utils::credentials::get_onenote_credentials(&credential_config).await?;
    let graph_client = GraphClient::new(token);

    let onedrive_path = audio_sync.onedrive_source_folder;

    let mut onedrive_source_path = onedrive_path
        .strip_suffix("/")
        .map(|x| x.to_owned())
        .unwrap_or(onedrive_path);
    if !onedrive_source_path.starts_with("/") {
        onedrive_source_path = format!("/{}", onedrive_source_path);
    }
    if onedrive_source_path == "/" {
        return Err(eyre!("Due to technical limitations the onenote source_dir cant be located at the drives root"));
    }

    let mut children = graph_client
        .me()
        .drive()
        .item_by_path(format!(":{}:", onedrive_source_path))
        .list_children()
        .select(&["name", "folder", "file"])
        .paging()
        .stream::<OneDriveChildren>()?;

    // check which files are already synced
    let mut files_to_sync = Vec::new();
    while let Some(result) = children.next().await {
        let res = result?;
        if !res.status().is_success() {
            return Err(eyre!(
                "Failed to read url {:?}: ({})",
                res.url(),
                res.status().as_u16()
            ));
        }
        let res = res.into_body()?;
        if res.folder.is_some() {
            continue;
        }
        let remote_file_extension = res.name.split(".").last().unwrap_or_default();
        if !audio_sync
            .permitted_file_types
            .contains(&remote_file_extension.to_owned())
        {
            log::debug!(
                "Skipped non audio file {}/{}",
                onedrive_source_path,
                res.name
            );
        }

        let name = format!("{}.link", res.name);
        if github_files
            .iter()
            .find(|x| {
                x.file_name()
                    .is_some_and(|x| x.to_str().is_some_and(|x| name == x))
            })
            .is_none()
        {
            files_to_sync.push(name);
        }
    }

    // syncing files
    for file in &files_to_sync {
        let git_target_file = git_target_path.join(&file);
        let onedrive_file = file.strip_suffix(".link").expect("Infallible").to_owned();
        let onedrive_path = format!("{}/{}", onedrive_source_path, onedrive_file);

        match std::fs::write(&git_target_file, format!("onedrive:{}", onedrive_path)) {
            Ok(_) => {
                log::info!(
                    "Wrote link onedrive:{} -> {:?}",
                    onedrive_path,
                    git_target_file
                );
            }
            Err(why) => {
                log::error!(
                    "Failed to write link onedrive:{} -> {:?}: {:?}",
                    onedrive_path,
                    git_target_file,
                    why
                );
            }
        }
    }

    // stage & commit changes
    wrap_git_command_error(&git_command_wrapper(&["add", "*"], &github_repo_root)?)?;
    wrap_git_command_error(&git_command_wrapper(
        &[
            "commit",
            "-m",
            &format!("add: {}", files_to_sync.join(",")),
            "--author",
            "Koro-sensei <koro-sensei@ansatsu-anime.com>",
        ],
        &github_repo_root,
    )?)?;

    Ok(())
}

#[derive(Debug, Clone)]
struct GitCommandOutput {
    status: ExitStatus,
    std_out: String,
    std_err: String,
    args: Vec<String>,
}
fn git_command_wrapper(args: &[&str], path: &Path) -> color_eyre::Result<GitCommandOutput> {
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

fn wrap_git_command_error(res: &GitCommandOutput) -> color_eyre::Result<()> {
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
