use dotenv::dotenv;
use teloxide::prelude::*;

mod bot_action;
mod config;
mod git;

#[tokio::main]
async fn main() {
    run().await;
}

async fn run() {
    pretty_env_logger::init();
    dotenv().ok();

    log::info!("Starting authenticate bot...");

    let bot = teloxide::Bot::from_env();

    // let archivist = Archivist { bot, repos };

    teloxide::repl(bot,   |message: Message, bot: Bot| async move {
        let secret = std::env::var("SECRET").unwrap_or("".to_string());
        let path = std::env::var("GIT_REPO").unwrap_or(".".to_string());
        let name = std::env::var("GIT_NAME").unwrap_or("archiver".to_string());
        let email = std::env::var("GIT_EMAIL").unwrap_or("archiver@mail.com".to_string());
        let repos = config::EnvironmentRepositoryFactory{
            repo: config::Repository::new(path, secret, name, email)
        };
        bot_action::bot_action(&repos, bot, message).await
    }).await;
}

