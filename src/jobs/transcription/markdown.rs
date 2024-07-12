use std::path::PathBuf;

use chrono::{Date, DateTime, Utc};

use crate::utils::config::Config;
use crate::utils::git;

pub(crate) struct CorrelatingFile {
    path: PathBuf,
    headlines: Vec<u32>,
}

pub(crate) async fn discorver_correlating_files(
    time: DateTime<Utc>,
    config: &Config,
) -> color_eyre::Result<Vec<CorrelatingFile>> {
    //TODO::
    //1. search all the files that have been changed the timerange (git log? -> find commits +/- n minutes)
    //2. extract only the affected headlines (affected headline = headline changed / new_headline / something under the headline has changed)
    //return the correlating files array
    unimplemented!();
}

async fn get_related_commits(config: &Config, time: &Date<Utc>) -> color_eyre::Result<Vec<String>> {
    //TODO: git log --pretty="format:%H %ct"
    //example output:
    //  <commit-hash>                           <UNIX-TIME>
    // b0edc6539e77ad73bdc26f1297137ec8ce33b808 1720786892
    // a1b06e8eca7ede4080e01e0ce20de85a7b70d5cf 1720784184
    // 602c07096d316622c40e001fbd00a9647fd8d4f3 1720737103
    // 09c61c03e78c5e8e1344f97d35c74018fe83507d 1720733077
    // 708033b856f6f4795cec84be33258896a78ac3a8 1720728602
    // cd77c2c3a92709ff0c1b608e02582a7bbbb3a6e9 1720723871
    // 2ed1f8f59afaeb70d42e8f7b8da82336c15ff19d 1720720206
    // b0ce59e83500f147825d2788f9f6593afa73cbcf 1720712972
    // 39ef6b0cf48365985eb8c9b308dcfe441d566430 1720628188
    // 3a7763c5da286ae7fce37fde71797cba5f39cf0a 1720623611
    // 887fdc6369428a794e8eb7875df85450faa8f882 1720608820

    let res = git::git_command_wrapper(&[], &config.git_directory, &config)?;
    git::wrap_git_command_error(&res)?;

    unimplemented!()
}
