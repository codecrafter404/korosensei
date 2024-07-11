use color_eyre::eyre::eyre;
use reqwest::header::HeaderMap;
use serde::Deserialize;

use super::config::CredentialConfig;

#[derive(Debug, Deserialize)]
pub struct OneNoteCredentialsResponse {
    pub scope: String,
    pub token: String,
}
pub async fn get_onenote_credentials(
    config: &CredentialConfig,
) -> color_eyre::Result<OneNoteCredentialsResponse> {
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.append(
        reqwest::header::AUTHORIZATION,
        config.onedrive_access_token_authorization.clone(),
    );
    let res = client
        .get(config.onedrive_access_token_url.clone())
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
    let text: OneNoteCredentialsResponse = res
        .json()
        .await
        .map_err(|e| eyre!("get_access_token: Failed to read response: {:?}", e))?;
    Ok(text)
}
