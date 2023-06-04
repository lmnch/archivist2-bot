use teloxide::prelude::*;
use async_trait::async_trait;
use std::error::{Error, self};

#[async_trait]
pub trait Authenticator {
    async fn is_authenticated(&self, chat: &ChatId) -> bool;
}


pub struct EnvironmentAuthenticator {
    secret: String,
}


impl EnvironmentAuthenticator {
    pub fn new() -> Self {
        Self {
            secret: std::env::var("SECRET").unwrap(),
        }
    }
}


#[async_trait]
impl Authenticator for EnvironmentAuthenticator {
    async fn is_authenticated(&self, chat: &ChatId) -> Result<bool, error::Error> {
        let auth_message: Option<Box<Message>> = self.bot.get_chat(chat).await?.pinned_message.clone();
        log::info!("[chat: {}] Current auth message: {:?}", chat, auth_message);

         
    }
}
