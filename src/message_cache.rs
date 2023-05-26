use std::collections::HashMap;
use teloxide::{prelude::*, net::Download};


pub trait MessageCache {
    fn put(&mut self, chatId: ChatId, message: Message);
    fn pop(&mut self, chatId: ChatId) -> Option<Message>;
}

pub struct InMemoryMessageCache {
    cache: HashMap<ChatId, Message>
}

impl InMemoryMessageCache {
    pub fn new() -> InMemoryMessageCache {
        InMemoryMessageCache {
            cache: HashMap::new()
        }
    }
}

impl MessageCache for InMemoryMessageCache {
    fn put(&mut self, chatId: ChatId, message: Message) {
        self.cache.insert(chatId, message);
    }

    fn pop(&mut self, chatId: ChatId) -> Option<Message> {
        self.cache.remove(&chatId)
    }
}
