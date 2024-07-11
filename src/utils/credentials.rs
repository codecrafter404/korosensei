use std::{
    ops::Deref,
    sync::{Mutex, OnceLock},
};

use color_eyre::eyre::eyre;
use reqwest::header::HeaderMap;
use serde::Deserialize;

use super::config::CredentialConfig;

#[derive(Debug, Clone, Deserialize)]
pub struct OneNoteCredentialsResponse {
    pub scope: String,
    pub token: String,
}

fn credentials_cache() -> &'static Mutex<Option<OneNoteCredentialsResponse>> {
    static ONENOTE_CREDENTIAL_CACHE: std::sync::OnceLock<
        Mutex<Option<OneNoteCredentialsResponse>>,
    > = OnceLock::new();
    ONENOTE_CREDENTIAL_CACHE.get_or_init(|| Mutex::new(None))
}

pub async fn get_onenote_credentials(
    config: &CredentialConfig,
) -> color_eyre::Result<OneNoteCredentialsResponse> {
    if let Some(x) = credentials_cache()
        .lock()
        .map_err(|x| eyre!("{:?}", x))?
        .deref()
    {
        return Ok(x.clone());
    }
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
    let res: OneNoteCredentialsResponse = res
        .json()
        .await
        .map_err(|e| eyre!("get_access_token: Failed to read response: {:?}", e))?;

    *(credentials_cache().lock().map_err(|x| eyre!("{:?}", x))?) = Some(res.clone());

    Ok(res)
}
