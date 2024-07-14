use std::{path::Path, process::ExitStatus};

use color_eyre::eyre::{eyre, OptionExt as _};
use graph_rs_sdk::{http::HttpResponseExt as _, GraphClient, ODataQuery as _};
use itertools::Itertools;
use serde::Deserialize;

use crate::utils::{
    config::Config,
    git::{check_out_create_branch, git_command_wrapper, wrap_git_command_error, GIT_AUTHOR},
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OneDriveChildren {
    name: String,
    // file: Option<OneDriveFile>,
    folder: Option<OneDriveFolder>,
    last_modified_date_time: String,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OneDriveChildrenVec {
    value: Vec<OneDriveChildren>,
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

    let github_repo_root = &config.git_directory;

    //TODO: validate that the branch exists

    check_out_create_branch(&audio_sync.git_branch, &config)?;
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
    let token = crate::utils::credentials::get_onedrive_credentials(&credential_config).await?;
    if !token.scope.contains("Files.Read") {
        return Err(eyre!("Access token didn't cover the scrope 'Files.Read'"));
    }
    let graph_client = GraphClient::new(token.token);

    let onedrive_path = audio_sync.onedrive_source_folder;

    let mut onedrive_source_path = onedrive_path
        .strip_suffix("/")
        .map(|x| x.to_owned())
        .unwrap_or(onedrive_path);
    if !onedrive_source_path.starts_with("/") {
        onedrive_source_path = format!("/{}", onedrive_source_path);
    }
    if onedrive_source_path == "/" {
        return Err(eyre!("Due to technical limitations the onedrive source_dir cant be located at the drives root"));
    }

    let children = graph_client
        .me()
        .drive()
        .item_by_path(format!(":{}:", onedrive_source_path))
        .list_children()
        .select(&["name", "folder", "file", "lastModifiedDateTime"])
        .paging()
        .json::<OneDriveChildrenVec>()
        .await?;

    let mut files_to_sync = vec![];

    // check which files are already synced
    for res in children.into_iter() {
        if !res.status().is_success() {
            return Err(eyre!(
                "Failed to read url {:?}: ({})",
                res.url(),
                res.status().as_u16()
            ));
        }
        let res = res.into_body()?;
        for res in res.value {
            if res.folder.is_some() {
                continue;
            }
            let remote_file_extension = res.name.split(".").last().unwrap_or_default();
            if !audio_sync
                .permitted_file_types
                .contains(&remote_file_extension.to_owned())
            {
                log::info!(
                    "Skipped non audio file {}/{}",
                    onedrive_source_path,
                    res.name
                );
                continue;
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
                files_to_sync.push((name, res.last_modified_date_time));
            }
        }
    }

    let mut synced_files = 0;
    // syncing files
    for (file, date) in &files_to_sync {
        let git_target_file = git_target_path.join(&file);
        let onedrive_file = file.strip_suffix(".link").expect("Infallible").to_owned();
        let onedrive_path = format!("{}/{}", onedrive_source_path, onedrive_file);
        let date = chrono::DateTime::parse_from_rfc3339(&date)?;

        match std::fs::write(
            &git_target_file,
            format!("onedrive:({}):{}", date.timestamp(), onedrive_path),
        ) {
            Ok(_) => {
                log::info!(
                    "Wrote link onedrive:{} -> {:?}",
                    onedrive_path,
                    git_target_file
                );
                synced_files += 1;
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

    if synced_files >= 1 {
        // stage & commit changes
        wrap_git_command_error(&git_command_wrapper(
            &["add", "*"],
            &github_repo_root,
            &config,
        )?)?;
        wrap_git_command_error(&git_command_wrapper(
            &[
                "commit",
                "-m",
                &format!(
                    "add: {}",
                    files_to_sync
                        .iter()
                        .map(|x| x.0.clone())
                        .collect_vec()
                        .join(",")
                ),
                "--author",
                GIT_AUTHOR,
            ],
            &github_repo_root,
            &config,
        )?)?;
        log::info!("Successfully commited {} links", synced_files);
    } else {
        log::info!("No files have been commited")
    }

    Ok(())
}
