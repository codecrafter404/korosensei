use color_eyre::eyre::{eyre, OptionExt};
use graph_rs_sdk::{Graph, ODataQuery};
use serde::Deserialize;
use worker::*;

#[event(scheduled)]
async fn scheduled(event: worker::ScheduledEvent, env: Env, context: ScheduleContext) {}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Children {
    name: String,
    file: Option<File>,
    folder: Option<Folder>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Folder {
    child_count: i32,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct File {
    mime_type: Option<String>,
}
async fn handle_scheduled_evnet(env: Env) -> color_eyre::Result<()> {
    let token = get_access_token(env.clone()).await?;
    let client = Graph::new(token);

    let path = env
        .kv("kv")?
        .get("onedrive_path")
        .text()
        .await
        .map_err(|e| color_eyre::eyre::eyre!("{:?}", e.to_string()))?
        .ok_or_eyre("Expected onedrive_path to be set")?;

    let path = path.strip_suffix("/").map(|x| x.to_owned()).unwrap_or(path);

    let children = client
        .me()
        .drive()
        .item_by_path(format!(":/{}:", path))
        .list_children()
        .select(&["name", "folder", "file"])
        .send()
        .await?;

    if !children.status().is_success() {
        return Err(eyre!(
            "Api returned {}: {:?}",
            children.status().as_u16(),
            children.text().await
        ));
    }
    let children: Vec<Children> = children.json().await?;

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
