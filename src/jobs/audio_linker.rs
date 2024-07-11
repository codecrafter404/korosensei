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

    //TODO: validate that the branch exists

    // TODO: list files

    let github_files = vec![];

    // OneDrive
    let token = crate::utils::credentials::get_onenote_credentials().await?;
    let graph_client = GraphClient::new(token);

    let onedrive_path = get_kv_key("onedrive_path").await?;

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
            .repos(github_repo[0], github_repo[1])
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
