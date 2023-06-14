use teloxide::prelude::*;
use async_trait::async_trait;
use std::{error::{Error, self}, string};

pub struct Authenticator {}

impl Authenticator {
    pub fn new() -> Authenticator {
        Authenticator {}
    }

    pub async fn get_auth(&self, bot: &Bot, chat: &ChatId) -> Result<Option<String>, teloxide::RequestError> {
        let auth_message: Option<Box<Message>> = bot.get_chat(chat.clone()).await?.pinned_message.clone();
        // log::info!("[chat: {}] Current auth message: {:?}", chat, auth_message);

        if auth_message.is_some() {
            let unwrapped = auth_message.unwrap();
            if unwrapped.text().is_some() {
                    let auth_text = unwrapped.text().unwrap();
                    let passed_secret = auth_text.replace("/auth ", "");
                    return Ok(Some(passed_secret));
            }
        }


        bot.send_message(chat.clone(), "Please authenticate first!")
                .await?;

            log::info!("[auth: {}] No authentication message found", chat);

            return Ok(None);

    }
}
