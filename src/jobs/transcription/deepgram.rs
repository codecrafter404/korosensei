use std::path::PathBuf;

use color_eyre::eyre::{eyre, OptionExt};
use deepgram::transcription::prerecorded::{
    audio_source::AudioSource,
    options::{self, OptionsBuilder},
    response::{Paragraph, TopicDetail},
};
use graph_rs_sdk::{GraphClient, ODataQuery};
use itertools::Itertools;
use serde::Deserialize;

use crate::utils::config::{Config, TranscriptionConfig};

use super::link::Link;

#[derive(Debug, Clone)]
pub struct TranscriptionResult {
    pub paragraphs: Vec<Paragraph>,
    pub summary: String,
    pub topics: Vec<TopicDetail>,
}

pub(crate) async fn transcribe_link(
    link: &Link,
    conf: &Config,
    deepgram: deepgram::Deepgram,
    graph: &GraphClient,
) -> color_eyre::Result<TranscriptionResult> {
    let source = get_source(link, conf, graph).await?;
    let options = OptionsBuilder::new()
        .model(options::Model::Nova2Meeting)
        .diarize(true)
        .detect_language(true)
        .summarize("v2")
        .topics(true)
        .smart_format(true)
        .punctuate(true)
        .paragraphs(true)
        .build();

    //TODO: finish & check if the crate has been updated yet

    let response = deepgram
        .transcription()
        .prerecorded(source, &options)
        .await?;
    let res = response
        .clone()
        .results
        .ok_or_eyre(format!("Expected results to be Some, got {:?}", response))?;
    let summary = res
        .clone()
        .summary
        .ok_or_eyre(format!("Expected summary to be enabled; got {:?}", res))?
        .short;

    let paragraphs = res
        .clone()
        .channels
        .first()
        .ok_or_eyre(format!(
            "Expected audio file to have at least one channel; got {:?}",
            res
        ))?
        .alternatives
        .clone()
        .into_iter()
        .max_by(|a, b| a.confidence.total_cmp(&b.confidence))
        .ok_or_eyre(format!("Expected to get min.1 alternative; got {:?}", res))?
        .paragraphs
        .clone()
        .ok_or_eyre(format!("Expected to get paragraphs, got {:?}", res))?
        .paragraphs;
    let topics = res
        .clone()
        .topics
        .ok_or_eyre(eyre!("Expected to get topics; got {:?}", res))?
        .segments
        .clone()
        .into_iter()
        .flat_map(|x| x.topics)
        .sorted_by(|a, b| b.confidence_score.total_cmp(&a.confidence_score))
        .collect_vec();

    Ok(TranscriptionResult {
        paragraphs,
        topics,
        summary,
    })
}
async fn get_source(
    link: &Link,
    config: &Config,
    graph: &GraphClient,
) -> color_eyre::Result<AudioSource> {
    let res = match &link.link_target {
        crate::jobs::transcription::link::LinkType::FileSytemLink(rel_path) => {
            let path = config
                .git_directory
                .join(rel_path.strip_prefix("/").unwrap_or(&rel_path));
            let file = tokio::fs::File::open(path).await?;
            AudioSource::from_buffer(file)
        }
        crate::jobs::transcription::link::LinkType::WebLink(link) => {
            AudioSource::from_url(link.clone())
        }
        crate::jobs::transcription::link::LinkType::OneDriveLink(link) => {
            AudioSource::from_url(get_onedrive_download_link(link.clone(), graph).await?)
        }
    };
    Ok(res)
}
#[derive(Debug, Deserialize)]
struct GraphResponse {
    #[serde(rename = "@microsoft.graph.downloadUrl")]
    download_url: String,
}
async fn get_onedrive_download_link(
    path: PathBuf,
    graph: &GraphClient,
) -> color_eyre::Result<reqwest::Url> {
    let path = path.strip_prefix("/").unwrap_or(&path);
    let file = graph
        .me()
        .drive()
        .item_by_path(format!(
            ":/{}:",
            path.to_str()
                .ok_or_eyre(format!("Expected path {:?} to be parsable", path))?
        ))
        .get_items()
        .select(&["@microsoft.graph.downloadUrl"])
        .send()
        .await?
        .json::<GraphResponse>()
        .await?;
    Ok(file.download_url.parse()?)
}
