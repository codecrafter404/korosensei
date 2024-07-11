use std::{
    path::Path,
    process::{ExitStatus, Output},
};

use color_eyre::eyre::{eyre, OptionExt as _};
use futures::StreamExt as _;
use graph_rs_sdk::{http::HttpResponseExt as _, GraphClient, ODataQuery as _};
use serde::Deserialize;

use crate::utils::config::Config;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OneDriveChildren {
    name: String,
    file: Option<OneDriveFile>,
    folder: Option<OneDriveFolder>,
}
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OneDriveFolder {
    child_count: i32,
}
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OneDriveFile {
    mime_type: Option<String>,
}
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct GithubTreeNode {
    path: String,
    mode: String,
    #[serde(rename = "type")]
    _type: String,
    sha: String,
    url: String,
    size: i32,
}
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct GithubTreeResponse {
    sha: String,
    url: String,
    tree: Vec<GithubTreeNode>,
}
async fn link_audio(config: &Config) -> color_eyre::Result<()> {
    let credential_config = config.credentials;
    let audio_sync = config
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

    let github_files = vec![];

    // OneDrive
    let token = crate::utils::credentials::get_onenote_credentials(&credential_config).await?;
    let graph_client = GraphClient::new(token);

    let onedrive_path = audio_sync.onedrive_source_folder;

    let mut onedrive_path = onedrive_path
        .strip_suffix("/")
        .map(|x| x.to_owned())
        .unwrap_or(onedrive_path);
    if !onedrive_path.starts_with("/") {
        onedrive_path = format!("/{}", onedrive_path);
    }

    let mut children = graph_client
        .me()
        .drive()
        .item_by_path(format!(":/{}:", onedrive_path))
        .list_children()
        .select(&["name", "folder", "file"])
        .paging()
        .stream::<OneDriveChildren>()?;

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
        let name = format!("{}.link", res.name);
        if !github_files.contains(&name) {
            files_to_sync.push(name);
        }
    }

    // syncing files
    for file in files_to_sync {
        let github_path = format!("{}/{}", github_repo_path, file);
        let onedrive_file = file.strip_suffix(".link").expect("Infallible").to_owned();
        let onedrive_path = format!("{}/{}", onedrive_path, onedrive_file);
        match github_client
            .repos(github_repo_root[0], github_repo_root[1])
            .create_file(
                github_path.clone(),
                format!("Synced file onedrive:{} -> {}", onedrive_path, github_path),
                format!("onedrive:{}", onedrive_path),
            )
            .send()
            .await
        {
            Ok(_) => {
                log::info!("Synced file onedrive:{} -> {}", onedrive_path, github_path);
            }
            Err(why) => {
                log::error!(
                    "Error while syncing file (onedrive:{} -> {}): {:?}",
                    onedrive_path,
                    github_path,
                    why
                );
            }
        }
    }
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
