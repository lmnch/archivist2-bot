use teloxide::prelude::*;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info("Starting authenticate bot...");

    let bot = Bot::from_env();

    teloxide::repl(bot, |message| async move {
        message.answer("Hello, World!").send().await?;
        respond(())
    })
}
