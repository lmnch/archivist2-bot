use regex::Regex;
use reqwest;
use async_trait::async_trait;


#[async_trait]
pub trait CommitMessageGenerator {
    async fn generate(&self)  -> String;
}


pub struct WhatTheCommitMessageGenerator {
}


impl WhatTheCommitMessageGenerator {
    pub fn new() -> WhatTheCommitMessageGenerator {
        WhatTheCommitMessageGenerator {}
    }
}


#[async_trait]
impl CommitMessageGenerator for WhatTheCommitMessageGenerator {

    async fn generate(&self) ->  String {
        let result = reqwest::get("https://whatthecommit.com/").await;
        if result.is_err(){
            return "".to_string();
        }

        let text_content =  result.unwrap().text().await;
        let regex = Regex::new(r#"(?m)id="content">\s*<p>(.+)\s*<\/p>"#).unwrap();
        if text_content.is_err(){
            return "".to_string();
        }

        regex.captures(text_content.unwrap().as_str()).map(|x| x.get(1).unwrap().as_str()).unwrap_or("").to_string()
    }

}







