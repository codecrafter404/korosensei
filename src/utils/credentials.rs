use color_eyre::eyre::eyre;
use reqwest::header::HeaderMap;

use super::config::CredentialConfig;

pub async fn get_onenote_credentials(config: CredentialConfig) -> color_eyre::Result<String> {
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.append(
        reqwest::header::AUTHORIZATION,
        config.onedrive_access_token_authorization,
    );
    let res = client
        .get(config.onedrive_access_token_url)
        .headers(headers)
        .send()
        .await?;
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
