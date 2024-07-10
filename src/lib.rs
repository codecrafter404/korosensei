use std::ops::Deref;

use color_eyre::{
    eyre::{eyre, OptionExt},
    owo_colors::OwoColorize,
};
use futures::StreamExt;
use graph_rs_sdk::{http::HttpResponseExt, Graph, GraphClient, ODataQuery};
use serde::Deserialize;
use worker::*;

#[event(scheduled)]
async fn scheduled(event: worker::ScheduledEvent, env: Env, context: ScheduleContext) {}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OneDriveChildren {
    name: String,
    file: Option<OneDriveFile>,
    folder: Option<OneDriveFolder>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OneDriveFolder {
    child_count: i32,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OneDriveFile {
    mime_type: Option<String>,
}

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
#[derive(Debug, Deserialize)]
struct GithubTreeResponse {
    sha: String,
    url: String,
    tree: Vec<GithubTreeNode>,
}
async fn handle_scheduled_evnet(env: Env) -> color_eyre::Result<()> {
    let kv = env.kv("kv")?;
    // GitHub
    let github_api_key = kv
        .get("github_api_key")
        .text()
        .await
        .map_err(|e| eyre!("{:?}", e))?
        .ok_or_eyre("expected github_api_key to be set")?;
    let github_repo = kv
        .get("github_repo")
        .text()
        .await
        .map_err(|e| eyre!("{:?}", e))?
        .ok_or_eyre("expected github_repo to be set")?;
    let github_repo = github_repo.split("/").collect::<Vec<_>>();
    if github_repo.len() != 2 {
        return Err(eyre!("Expected github_repo in format owner/repository"));
    }
    let github_repo_branch = kv
        .get("github_repo_branch")
        .text()
        .await
        .map_err(|e| eyre!("{:?}", e))?
        .ok_or_eyre("expected github_repo_branch to be set")?;

    let client = octocrab::OctocrabBuilder::new()
        .personal_token(github_api_key)
        .build()?;
    let mut current_branch_list = client
        .repos(github_repo[0], github_repo[1])
        .list_branches()
        .per_page(100)
        .send()
        .await?;
    let mut branches = current_branch_list.take_items();
    while let Ok(Some(mut new_page)) = client.get_page(&current_branch_list.next).await {
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

    let res: GithubTreeResponse = client
        .get(
            format!(
                "/repos/{}/{}/git/trees/{}",
                github_repo[0], github_repo[1], github_repo_branch
            ),
            Some(""),
        )
        .await?;

    // OneDrive
    let token = get_access_token(env.clone()).await?;
    let client = GraphClient::new(token);

    let path = env
        .kv("kv")?
        .get("onedrive_path")
        .text()
        .await
        .map_err(|e| color_eyre::eyre::eyre!("{:?}", e.to_string()))?
        .ok_or_eyre("Expected onedrive_path to be set")?;

    let path = path.strip_suffix("/").map(|x| x.to_owned()).unwrap_or(path);

    let mut children = client
        .me()
        .drive()
        .item_by_path(format!(":/{}:", path))
        .list_children()
        .select(&["name", "folder", "file"])
        .paging()
        .stream::<OneDriveChildren>()?;
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
    }
    unimplemented!()
}

async fn get_access_token(env: Env) -> worker::Result<String> {
    let kv = env.kv("kv")?;
    let url = match kv.get("access_token_url").text().await? {
        Some(x) => x,
        None => {
            return Err(worker::Error::RustError(
                "expected access_token_url to be set".into(),
            ))
        }
    };
    let authorization_header = match kv.get("access_token_authorization").text().await? {
        Some(x) => x,
        None => {
            return Err(worker::Error::RustError(
                "expected access_token_authorization to be set".into(),
            ))
        }
    };
    let client = reqwest::Client::new();
    let res = client
        .get(url)
        .header("Authorization", authorization_header)
        .send()
        .await
        .map_err(|e| worker::Error::RustError(format!("get_access_token_res: {:?}", e).into()))?;
    if !res.status().is_success() {
        return Err(worker::Error::RustError(format!(
            "get_access_token_res; access_token_url returned {}: {:?}",
            res.status().as_u16(),
            res.text().await
        )));
    }
    let text = res.text().await.map_err(|e| {
        worker::Error::RustError(format!(
            "get_access_token: Failed to read response text: {:?}",
            e
        ))
    })?;
    Ok(text)
}
