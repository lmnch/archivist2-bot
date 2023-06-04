use archivist::Archivist;
use dotenv::dotenv;
use teloxide::{dispatching::dialogue::InMemStorage, prelude::*};

use crate::archivist::{BilloArchivist};

// mod bot_action;
mod archivist;
mod categorizer;
mod commit_messages;
mod config;
mod message_cache;
mod path_matcher;
mod publisher;
mod authenticate;

type UploadDialogue = Dialogue<State, InMemStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,
    ReceivedCaption(String),
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    log::info!("Starting authenticate bot...");

    let bot = teloxide::Bot::from_env();
    log::info!("Starting bot...");

    Dispatcher::builder(
        bot, 
        Update::filter_message()
            .enter_dialogue::<Message, InMemStorage<State>, State>()
            .branch(dptree::case![State::Start].endpoint(receive_caption))
            .branch(dptree::case![State::ReceivedCaption(caption)].endpoint(receive_document))
    )
    .dependencies(dptree::deps![InMemStorage::<State>::new()])
    .enable_ctrlc_handler()
    .build()
    .dispatch()
    .await;
}


async fn receive_caption(bot: Bot, dialogue: UploadDialogue, msg: Message) -> HandlerResult {
    match msg.text() {
        Some(text) => {
            dialogue.update(State::ReceivedCaption(text.into())).await?;
            log::info!("Received text: {}", text);
        }
        None => {
            log::info!("No text in message");
        }
    }
    match msg.document() {
        // Upload directly if only document
        Some(doc) => {
            let archivist = BilloArchivist::new(bot);
            if msg.caption().is_some(){
                let cap = msg.caption().unwrap();
                archivist.upload_document(msg.chat.id, doc, Some(&cap.to_string())).await?;
            } else {
                archivist.upload_document(msg.chat.id, doc, None).await?;
            }
        }
        None => {
            log::info!("No document in message");
        }
    }

    Ok(())
}


async fn receive_document(bot: Bot, dialogue: UploadDialogue, caption: String, msg: Message) -> HandlerResult {
    match msg.document() {
        Some(doc) => {
            let archivist = BilloArchivist::new(bot);
            archivist.upload_document(msg.chat.id, doc, Some(&caption)).await?;
        }
        None => {
            log::info!("No document in message");
        }
    }
    dialogue.exit().await?;

    Ok(())
}
