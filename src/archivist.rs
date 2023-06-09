use std::{
    collections::HashMap,
    path::Path,
    sync::{Arc, Mutex},
};
use async_trait::async_trait;
use teloxide::{net::Download, prelude::*, types::Document};
use tokio::fs;

use crate::{
    categorizer::{self, Categorizer, RepoBasedCategorizer},
    commit_messages::{self, CommitMessageGenerator, WhatTheCommitMessageGenerator},
    config::{EnvironmentRepositoryFactory, Repository, RepositoryFactory, JsonRepositoryFactory},
    message_cache::{MessageCache, SyncedInMemoryMessageCache},
    path_matcher::{self, Matcher},
    publisher::{self, GitPublisher, Publisher},
    authenticate::Authenticator,
};


pub trait Archivist {
    fn trigger_upload_document(
        &self,
        chat: ChatId,
        document: &Document,
        caption: Option<&String>,
    ) -> ResponseResult<()>;
}

pub struct ArchivistImpl<
    T: RepositoryFactory,
    P: publisher::Publisher,
    C: categorizer::Categorizer,
    M: commit_messages::CommitMessageGenerator,
> {
    pub bot: Bot,
    pub repos: T,
    pub publisher: P,
    pub categorizer: C,
    pub matcher: path_matcher::Matcher<
        path_matcher::AddRule<path_matcher::LatestRule<path_matcher::DefaultRule>>,
    >,
    pub message_generator: M,
    pub authenticator: Authenticator,
}

pub type BilloArchivist = ArchivistImpl<
    JsonRepositoryFactory,
    GitPublisher,
    RepoBasedCategorizer,
    WhatTheCommitMessageGenerator,
>;

impl BilloArchivist {
    pub fn new(bot: Bot) -> BilloArchivist {
        // let secret = std::env::var("SECRET").unwrap_or("".to_string());
        // let path = std::env::var("GIT_REPO").unwrap_or(".".to_string());
        let path = std::env::var("GIT_REPO_CONFIG").unwrap_or(".".to_string());
        let name = std::env::var("GIT_NAME").unwrap_or("archiver".to_string());
        let email = std::env::var("GIT_EMAIL").unwrap_or("archiver@mail.com".to_string());
        let ssh_key = std::env::var("SSH_KEY").unwrap_or("".to_string());

        log::info!("Starting with...");
        // log::info!("SECRET:    {}", secret);
        // log::info!("GIT_REPO:  {}", path);
        log::info!("GIT_REPO_CONFIG:  {}", path);
        log::info!("GIT_NAME:  {}", name);
        log::info!("GIT_EMAIL: {}", email);
        log::info!("SSH_KEY:   {}", ssh_key);


        let repos = JsonRepositoryFactory::new(&path, &name, &email);

        let publisher = publisher::GitPublisher::new(ssh_key);
        let categori = categorizer::RepoBasedCategorizer::new();

        ArchivistImpl {
            bot,
            repos,
            publisher,
            matcher: Matcher::new(),
            categorizer: RepoBasedCategorizer::new(),
            message_generator: WhatTheCommitMessageGenerator::new(),
            authenticator: Authenticator::new(),
        }
    }
}


impl<
        T: RepositoryFactory,
        P: Publisher,
        C: Categorizer,
        M: CommitMessageGenerator,
    >  ArchivistImpl<T, P, C, M>{
        pub async fn upload_document(
            &self,
            chat: ChatId,
            document: &Document,
            caption: Option<&String>,
        ) -> ResponseResult<()> {
            
            let passed_secret = self.authenticator.get_auth(&self.bot, &chat).await?;
            if passed_secret.is_none() {
                return Ok(());
            }
            let repo = self.repos.get_repository(&passed_secret.unwrap());
    
            if repo.is_none() {
                self.bot
                    .send_message(chat, "Incorrect authentication token!")
                    .await?;
    
                log::info!("[chat: {}] Incorrect authentication token", chat);
    
                return Ok(());
            }
    
            // Write file to disk
            let file_meta = document.file.clone();
            let file = self.bot.get_file(file_meta.id).await?;
    
            // Pull changes upfront
            let pull_result = self.publisher.update_files(repo.unwrap());
            if pull_result.is_err() {
                let error = pull_result.err().unwrap();
                self.bot
                    .send_message(chat, format!("Pull failed: {}", &error))
                    .await?;
                log::error!(
                    "[chat: {}] Pull failed: {}",
                    chat,
                    &error                    
                );
                return Ok(());
            }
    
            log::info!(
                "[chat: {}] Pushing file {:?} to repo at {}",
                chat,
                file,
                repo.unwrap().path()
            );
            let dest = Path::new(repo.unwrap().path());
            let matching_template;
            if caption.is_some() {
                matching_template = self.categorizer.categorize(
                    Some(caption.unwrap().as_str()),
                    categorizer::CategorizationContext::new(repo.unwrap(), chat.0),
                );
            } else {
                matching_template = self.categorizer.categorize(
                    None,
                    categorizer::CategorizationContext::new(repo.unwrap(), chat.0),
                );
            }
            let target = self.matcher.resolve(&repo.unwrap(), matching_template);
            let rel_path = Path::new(&target);
            let path = dest.join(rel_path.clone());
            if path.parent().is_some() && !path.parent().unwrap().exists() {
                fs::create_dir_all(path.parent().unwrap()).await?;
                log::info!(
                    "[chat: {}] Created directory {:?}",
                    chat,
                    path.parent().unwrap()
                );
            }
            let mut dst = fs::File::create(path).await?;
            log::info!("[chat: {}] Created file at {:?}", chat, target);
    
            let downloaded = self.bot.download_file(&file.path, &mut dst).await?;
            log::info!("[chat: {}] Downloaded file {:?}", chat, downloaded);
            self.bot
                .send_message(chat, format!("File stored at {}", target.to_string()))
                .await?;
    
            let commit_msg = self.message_generator.generate().await;
            let commit = self
                .publisher
                .publish_file(repo.unwrap(), rel_path, &commit_msg);
            log::info!("[chat: {}] Committed file {:?}", chat, commit);
            if commit.is_ok() {
                self.bot
                    .send_message(chat, format!("Commit: {}", commit.unwrap()))
                    .await?;
            } else {
                self.bot
                    .send_message(
                        chat,
                        format!("Error during commit: {}", commit.err().unwrap()),
                    )
                    .await?;
            }
    
            Ok(())
        }
    }

impl<
        T: RepositoryFactory,
        P: Publisher,
        C: Categorizer,
        M: CommitMessageGenerator,
    > Archivist for ArchivistImpl<T, P, C, M>
{
    fn trigger_upload_document(
        &self,
        chat: ChatId,
        document: &Document,
        caption: Option<&String>,
    ) -> ResponseResult<()> {
        self.upload_document(chat, document, caption);
        Ok(())
    }
}
