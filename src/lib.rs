use color_eyre::eyre::{eyre, OptionExt};
use futures::StreamExt;
use graph_rs_sdk::{http::HttpResponseExt, GraphClient, ODataQuery};
use serde::Deserialize;
use worker::*;

#[event(scheduled)]
async fn scheduled(_event: worker::ScheduledEvent, env: Env, _context: ScheduleContext) {
    match handle_scheduled_event(env).await {
        Ok(_) => {}
        Err(why) => {
            console_error!("Error while handling scheduled event: {:#?}", why);
        }
    }
}
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
async fn get_kv_key(env: &Env, key: &str) -> color_eyre::Result<String> {
    let kv = env.kv("kv")?;
    let res = kv
        .get(key)
        .text()
        .await
        .map_err(|e| eyre!("{:?}", e))?
        .ok_or_eyre(format!("expected {} to be set", key))?;

    Ok(res)
}
async fn handle_scheduled_event(env: Env) -> color_eyre::Result<()> {
    // GitHub
    let github_repo_path = get_kv_key(&env, "github_repo_path").await?;
    let github_repo_path = github_repo_path
        .strip_prefix("/")
        .unwrap_or(&github_repo_path)
        .to_owned();

    let github_repo_path = github_repo_path
        .strip_suffix("/")
        .unwrap_or(&github_repo_path)
        .to_owned();

    let github_api_key = get_kv_key(&env, "github_api_key").await?;
    let github_repo = get_kv_key(&env, "github_repo").await?;
    let github_repo = github_repo.split("/").collect::<Vec<_>>();
    if github_repo.len() != 2 {
        return Err(eyre!("Expected github_repo in format owner/repository"));
    }
    let github_repo_branch = get_kv_key(&env, "github_repo_branch").await?;

    let github_client = octocrab::OctocrabBuilder::new()
        .personal_token(github_api_key)
        .build()?;
    let mut current_branch_list = github_client
        .repos(github_repo[0], github_repo[1])
        .list_branches()
        .per_page(100)
        .send()
        .await?;
    let mut branches = current_branch_list.take_items();
    while let Ok(Some(mut new_page)) = github_client.get_page(&current_branch_list.next).await {
        branches.extend(new_page.take_items());
        current_branch_list = new_page;
    }
    if !branches
        .iter()
        .find(|x| x.name == github_repo_branch)
        .is_none()
    {
        return Err(eyre!("Branch {} doesn't exist", github_repo_branch));
    }

    let res: GithubTreeResponse = github_client
        .get(
            format!(
                "/repos/{}/{}/git/trees/{}",
                github_repo[0], github_repo[1], github_repo_branch
            ),
            Some("?recursive=1"),
        )
        .await?;

    let github_files = res
        .tree
        .into_iter()
        .filter(|x| x._type == "blob" && x.path.starts_with(&github_repo_path))
        .map(|x| {
            x.path
                .strip_prefix(&format!("{}/", github_repo_path))
                .ok_or_eyre(format!(
                    "Expect {} to have prefix {}",
                    x.path,
                    format!("{}/", github_repo_path)
                ))
                .map(|x| x.to_owned())
        })
        .collect::<color_eyre::Result<Vec<String>>>()?;

    // OneDrive
    let token = get_access_token(env.clone()).await?;
    let graph_client = GraphClient::new(token);

    let onedrive_path = get_kv_key(&env, "onedrive_path").await?;

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
                console_log!("Synced file onedrive:{} -> {}", onedrive_path, github_path);
            }
            Err(why) => {
                console_error!(
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

async fn get_access_token(env: Env) -> color_eyre::Result<String> {
    let url = get_kv_key(&env, "onedrive_access_token_url").await?;
    let authorization_header = get_kv_key(&env, "onedrive_access_token_authorization").await?;
    let client = reqwest::Client::new();
    let res = client
        .get(url)
        .header("Authorization", authorization_header)
        .send()
        .await
        .map_err(|e| worker::Error::RustError(format!("get_access_token_res: {:?}", e).into()))?;
    if !res.status().is_success() {
        return Err(eyre!(
            "get_access_token_res; access_token_url returned {}: {:?}",
            res.status().as_u16(),
            res.text().await
        ));
    }
    let text = res
        .text()
        .await
        .map_err(|e| eyre!("get_access_token: Failed to read response text: {:?}", e))?;
    Ok(text)
}
