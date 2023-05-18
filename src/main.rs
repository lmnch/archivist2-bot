use dotenv::dotenv;
use teloxide::prelude::*;

// mod bot_action;
mod config;
mod archivist;
mod publisher;
mod categorizer;

#[tokio::main]
async fn main() {
    run().await;
}

async fn run() {
    pretty_env_logger::init();
    dotenv().ok();

    log::info!("Starting authenticate bot...");

    let bot = teloxide::Bot::from_env();  
    
    teloxide::repl(bot, |bot: Bot, m: Message| async move { 
        let secret = std::env::var("SECRET").unwrap_or("".to_string());
        let path = std::env::var("GIT_REPO").unwrap_or(".".to_string());
        let name = std::env::var("GIT_NAME").unwrap_or("archiver".to_string());
        let email = std::env::var("GIT_EMAIL").unwrap_or("archiver@mail.com".to_string());
        let ssh_key = std::env::var("SSH_KEY").unwrap_or("".to_string());

        let repos = config::EnvironmentRepositoryFactory{
            repo: config::Repository::new(path, secret, name, email)
        };

        let publisher = publisher::GitPublisher::new(ssh_key);

        let categori = categorizer::ExactPathCategorizer::new();

        let archivist = archivist::Archivist { bot, repos, publisher, categorizer: categori  };

        archivist.answer(m).await?;
        Ok(()) 
    }).await;
}

