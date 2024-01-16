use std::collections::HashMap;

use crate::protocol::{ErrorKind, ErrorMessage, MessageContext, MessageReceiver};

use super::sender::MaelstromServerMessageSender;

pub struct MaelstromServerMessageHandler {
    msg_handlers: HashMap<String, usize>,
    handlers: Vec<Box<dyn MessageReceiver<MaelstromServerMessageSender>>>,
}

impl MaelstromServerMessageHandler {
    pub fn new() -> Self {
        Self {
            msg_handlers: HashMap::new(),
            handlers: Vec::new(),
        }
    }

    #[allow(private_bounds)]
    pub fn register_handler<T>(&mut self)
    where
        T: MessageReceiver<MaelstromServerMessageSender> + 'static,
    {
        let handle_idx = self.handlers.len();
        self.handlers.push(Box::new(T::new()));

        let msg_types = T::get_handled_messages();
        for msg_type in msg_types {
            self.msg_handlers.insert(msg_type.to_owned(), handle_idx);
        }
    }

    pub fn handle_message(
        &mut self,
        ctx: &MessageContext<MaelstromServerMessageSender>,
    ) -> Result<(), ErrorMessage> {
        let kind = ctx.message_kind();
        if let Some(&handler_idx) = self.msg_handlers.get(kind) {
            self.handlers[handler_idx].handle(ctx)
        } else {
            Err(ErrorMessage::new(
                ErrorKind::NotSupported,
                &format!("message type {kind} not supported"),
            ))
        }
    }
}
