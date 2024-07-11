mod jobs;
mod utils;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    env_logger::init();
    color_eyre::install()?;

    return Ok(());
}

async fn get_access_token() -> color_eyre::Result<String> {
    unimplemented!();
    // let url = get_kv_key("onedrive_access_token_url").await?;
    // let authorization_header = get_kv_key("onedrive_access_token_authorization").await?;
    // let client = reqwest::Client::new();
    // let res = client
    //     .get(url)
    //     .header("Authorization", authorization_header)
    //     .send()
    //     .await
    //     .map_err(|e| worker::Error::RustError(format!("get_access_token_res: {:?}", e).into()))?;
    // if !res.status().is_success() {
    //     return Err(eyre!(
    //         "get_access_token_res; access_token_url returned {}: {:?}",
    //         res.status().as_u16(),
    //         res.text().await
    //     ));
    // }
    // let text = res
    //     .text()
    //     .await
    //     .map_err(|e| eyre!("get_access_token: Failed to read response text: {:?}", e))?;
    // Ok(text)
}
