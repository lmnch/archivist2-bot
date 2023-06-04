use std::collections::HashMap;
use teloxide::prelude::*;
use teloxide::types::Me;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;

pub trait MessageCache {
    fn lock_chat(&mut self, chat_id: ChatId);
    fn put(&mut self, chat_id: ChatId, message: Message);
    fn pop(&mut self, chat_id: ChatId) -> Option<Message>;
}


pub struct InMemoryMessageCache {
    cache: HashMap<ChatId, Message>
}

impl MessageCache for InMemoryMessageCache {
    fn lock_chat(&mut self, chat_id: ChatId) {
    }

    fn put(&mut self, chat_id: ChatId, message: Message) {
        log::info!("[chat: {}] Caching message: {}", chat_id, message.text().unwrap());
        self.cache.insert(chat_id, message);
    }

    fn pop(&mut self, chat_id: ChatId) -> Option<Message> {
        let result= self.cache.remove(&chat_id);
        if result.is_none() {
            log::info!("[chat: {}] No message found in cache", chat_id);
            return None;
        }else{
            log::info!("[chat: {}] Removed message from cache {}", chat_id, result.as_ref().unwrap().text().unwrap());
            return Some(result.unwrap());
        }
    }
}

impl InMemoryMessageCache {
    pub fn new() -> InMemoryMessageCache {
        InMemoryMessageCache {
            cache: HashMap::new()
        }
    }
}


pub struct SyncedInMemoryMessageCache {
    cache: Arc<Mutex<HashMap<ChatId, Message>>>
}

impl MessageCache for SyncedInMemoryMessageCache {
    fn lock_chat(&mut self, chat_id: ChatId) {
    }

    fn put(&mut self, chat_id: ChatId, message: Message) {
        log::info!("[chat: {}] Caching message: {}", chat_id, message.text().unwrap());
        self.cache.lock().unwrap().insert(chat_id, message);
    }

    fn pop(&mut self, chat_id: ChatId) -> Option<Message> {
        let result= self.cache.lock().unwrap().remove(&chat_id);
        if result.is_none() {
            log::info!("[chat: {}] No message found in cache", chat_id);
            return None;
        }else{
            log::info!("[chat: {}] Removed message from cache {}", chat_id, result.as_ref().unwrap().text().unwrap());
            return Some(result.unwrap());
        }
    }
}

impl SyncedInMemoryMessageCache {
    pub fn new(cache: Arc<Mutex<HashMap<ChatId, Message>>>) -> SyncedInMemoryMessageCache {
        SyncedInMemoryMessageCache { cache }
    }
}



pub struct AsyncInMemoryMessageCache {
    cache: HashMap<ChatId, (Sender<Message>, Receiver<Message>)>
}

impl AsyncInMemoryMessageCache {
    pub fn new() -> AsyncInMemoryMessageCache {
        AsyncInMemoryMessageCache {
            cache: HashMap::new()
        }
    }
}

impl MessageCache for AsyncInMemoryMessageCache {
    fn lock_chat(&mut self, chat_id: ChatId) {
        self.cache.insert(chat_id, mpsc::channel()); 
    }

    fn put(&mut self, chat_id: ChatId, message: Message) {
        log::info!("[chat: {}] Caching message: {}", chat_id, message.text().unwrap());
        self.cache.get(&chat_id).unwrap().0.send(message);
    }

    fn pop(&mut self, chat_id: ChatId) -> Option<Message> {
        let result= self.cache.get(&chat_id).unwrap().1.recv();
        if result.is_err() {
            log::info!("[chat: {}] No message found in cache", chat_id);
            return None;
        }else{
            log::info!("[chat: {}] Removed message from cache {}", chat_id, result.as_ref().unwrap().text().unwrap());
            return Some(result.unwrap());
        }
    }
}
