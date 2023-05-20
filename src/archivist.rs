
use teloxide::{prelude::*, net::Download};
use tokio::fs;
use std::path::Path;

use crate::{config::RepositoryFactory, publisher, categorizer, path_matcher, commit_messages};


pub struct Archivist<T: RepositoryFactory, P: publisher::Publisher, C: categorizer::Categorizer, M: commit_messages::CommitMessageGenerator> {
    pub bot: Bot,
    pub repos: T,
    pub publisher: P,
    pub categorizer: C,
    pub matcher: path_matcher::Matcher<path_matcher::AddRule<path_matcher::LatestRule<path_matcher::DefaultRule>>>,
    pub message_generator: M, 
}


impl<T: RepositoryFactory, P: publisher::Publisher, C:categorizer::Categorizer, M: commit_messages::CommitMessageGenerator> Archivist<T, P, C, M> {    
    pub async fn answer(&self, msg: Message)  -> ResponseResult<()> {
        if msg.text().is_some() && msg.text().unwrap().starts_with("/auth") {
            // Unpin old auth messages
            self.bot.unpin_all_chat_messages(msg.chat.id).await?;
            self.bot.pin_chat_message(msg.chat.id, msg.id).await?;
            self.bot.send_message(msg.chat.id, "Credentials stored!").await?;

            println!("[chat: {}] Stored authentication credential", msg.chat.id);

            return Ok(())
    }else{
        let auth_message: Option<Box<Message>> = self.bot.get_chat(msg.chat.id).await?.pinned_message.clone();
        if !auth_message.is_some() {
            self.bot.send_message(msg.chat.id, "Please authenticate first!").await?;

            print!("[chat: {}] No authentication message found", msg.chat.id);

            return Ok(());
        }
        
        let auth = auth_message.unwrap();
        let auth_text = auth.text().unwrap(); 
        let passed_secret = auth_text.replace("/auth ", "");
        let repo = self.repos.get_repository(&passed_secret);

        if repo.is_none(){
            self.bot.send_message(msg.chat.id, "Incorrect authentication token!").await?;

            print!("[chat: {}] Incorrect authentication token", msg.chat.id);

            return Ok(())
        }

        // Could be also a file to be stored
        if msg.document().is_some() {
            // Write file to disk
            let file_meta = msg.document().unwrap().file.clone();
            let file = self.bot.get_file(file_meta.id).await?;

            // Pull changes upfront
            let pull_result = self.publisher.update_files(repo.unwrap());
            if pull_result.is_err() {
                self.bot.send_message(msg.chat.id, format!("Pull failed: {}", pull_result.err().unwrap())).await?;
                return Ok(());
            }

            // Get destinated location
            println!("[chat: {}] Pushing file {:?} to repo at {}", msg.chat.id, file, repo.unwrap().path());
            let dest = Path::new(repo.unwrap().path());
            let matching_template = self.categorizer.categorize(msg.caption(), categorizer::CategorizationContext::new(repo.unwrap(), msg.chat.id.0));
            let target = self.matcher.resolve(&repo.unwrap(), matching_template);
            let rel_path = Path::new(&target);
            let path = dest.join(rel_path.clone());
            if path.parent().is_some() && !path.parent().unwrap().exists() {
                    fs::create_dir_all(path.parent().unwrap()).await?;
                    println!("[chat: {}] Created directory {:?}", msg.chat.id, path.parent().unwrap());
            }
            let mut dst = fs::File::create(path).await?;
            println!("[chat: {}] Created file at {:?}", msg.chat.id, target);

            let downloaded = self.bot.download_file(&file.path, &mut dst).await?;
            println!("[chat: {}] Downloaded file {:?}", msg.chat.id, downloaded);
            self.bot.send_message(msg.chat.id, format!("File stored at {}", target.to_string())).await?;
            
            let commit_msg = self.message_generator.generate().await;
            let commit = self.publisher.publish_file(repo.unwrap(), rel_path, &commit_msg);
            println!("[chat: {}] Committed file {:?}", msg.chat.id, commit);
            if commit.is_ok() {
                self.bot.send_message(msg.chat.id, format!("Commit: {}", commit.unwrap())).await?;
            }else{
                self.bot.send_message(msg.chat.id, format!("Error during commit: {}", commit.err().unwrap())).await?;
            }
        }
    }
    Ok(())
    }
}
