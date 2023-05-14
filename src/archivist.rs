
use teloxide::{prelude::*, net::Download};
use tokio::fs;
use std::{path::Path, sync::Arc};

use crate::{config::RepositoryFactory, git};

struct Archivist {
    bot: Bot,
    repos: dyn RepositoryFactory
}


impl Archivist {
    async fn init(&self) {
        teloxide::repl(self.bot, |m: Message| async move { 
            self.reply(m); 
            Ok(()) 
        });
    }
    

    fn reply(&self, m: Message){

    }


    async fn answer(msg: Message)  -> ResponseResult<()> {
        if msg.text().is_some() && msg.text().unwrap().starts_with("/auth") {
            // Unpin old auth messages
            self.bot.unpin_all_chat_messages(msg.chat.id).await?;
            self.bot.pin_chat_message(msg.chat.id, msg.id).await?;
            self.bot.send_message(msg.chat.id, "Credentials stored!").await?;

            return Ok(())
    }else{
        let auth_message: Option<Box<Message>> = self.bot.get_chat(msg.chat.id).await?.pinned_message.clone();
        if !auth_message.is_some() {
            self.bot.send_message(msg.chat.id, "Please authenticate first!").await?;
            return Ok(());
        }
        
        let auth = auth_message.unwrap();
        let auth_text = auth.text().unwrap(); 
        let passed_secret = auth_text.replace("/auth ", "");
        let repo = self.repos.get_repository(&passed_secret);

        if repo.is_none(){
            self.bot.send_message(msg.chat.id, "Incorrect authentication token!");
            return Ok(())
        }

        // Could be also a file to be stored
        if msg.document().is_some() {
            // Write file to disk
            let file_meta = msg.document().unwrap().file.clone();
            println!("File id is: {}", file_meta.unique_id);
            let file = self.bot.get_file(file_meta.id).await?;

            // Get destinated location
            let dest = repo.unwrap().path().clone();
            let dst_name = msg.caption().unwrap_or("tmp.pdf");
            let rel_path = Path::new(dst_name);
            let path_str = dest + "/" + dst_name;
            let path = Path::new(&path_str);
            if path.parent().is_some() && !path.parent().unwrap().exists() {
                    fs::create_dir_all(path.parent().unwrap()).await?;
            }
            let mut dst = fs::File::create(path).await?;

            let downloaded = self.bot.download_file(&file.path, &mut dst).await?;
            println!("File: {:?}", downloaded);
            self.bot.send_message(msg.chat.id, "File stored at ".to_string() + path.to_str().unwrap()).await?;

            let commit = git::add_and_commit(repo.unwrap(), rel_path);
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

unsafe impl Sync for Archivist {}