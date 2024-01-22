use std::collections::HashMap;

use crate::protocol::{ErrorKind, ErrorMessage, MessageContext, MessageReceiver};

use super::sender::MaelstromServerMessageSender;

pub struct MaelstromServerMessageHandler {
    msg_handlers: HashMap<String, Vec<usize>>,
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
            let k = msg_type.to_owned();
            if let Some(idxs) = self.msg_handlers.get_mut(&k) {
                idxs.push(handle_idx);
            } else {
                self.msg_handlers.insert(k, vec![handle_idx]);
            }
        }
    }

    pub fn handle_message(
        &mut self,
        ctx: &MessageContext<MaelstromServerMessageSender>,
    ) -> Result<(), ErrorMessage> {
        let kind = ctx.message_kind();
        if let Some(handler_idxs) = self.msg_handlers.get(kind) {
            for handler_idx in handler_idxs {
                self.handlers[*handler_idx].handle(ctx)?;
            }
            Ok(())
        } else {
            Err(ErrorMessage::new(
                ErrorKind::NotSupported,
                &format!("message type {kind} not supported"),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    use super::*;
    use crate::protocol::{Message, MessageBody, MessageContent};

    #[test]
    fn test_single_handler() {
        #[derive(Default)]
        struct TestHandler;

        impl MessageReceiver<MaelstromServerMessageSender> for TestHandler {
            fn new() -> Self {
                Self
            }

            fn get_handled_messages() -> impl Iterator<Item = &'static str> {
                ["test"].into_iter()
            }

            fn handle(
                &mut self,
                _ctx: &MessageContext<MaelstromServerMessageSender>,
            ) -> Result<(), ErrorMessage> {
                Err(ErrorMessage::new(ErrorKind::Crash, "hello there"))
            }
        }

        let mut handler = MaelstromServerMessageHandler::new();
        handler.register_handler::<TestHandler>();

        let msg = Message {
            src: None,
            dest: None,
            body: MessageBody {
                msg_id: Some(1),
                in_reply_to: None,
                content: MessageContent {
                    kind: "test".to_string(),
                    data: Default::default(),
                },
            },
        };

        let sender = MaelstromServerMessageSender::new();
        let ctx = MessageContext::new(Some(&msg), &[], &sender);
        let res = handler.handle_message(&ctx);

        // The easiest way to check if the handler was called is to use the error result, because we cannot downcast from `dyn MessageReceiver` to `TestHandler`
        assert!(
            res.is_err_and(
                |x| x.code() == usize::from(ErrorKind::Crash) && x.text() == "hello there"
            )
        );
    }

    #[test]
    fn test_multiple_handlers_success() {
        #[derive(Default)]
        struct TestHandler1;

        #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
        struct TestResponseMessage1 {
            foo: String,
        }

        impl MessageReceiver<MaelstromServerMessageSender> for TestHandler1 {
            fn new() -> Self {
                Self
            }

            fn get_handled_messages() -> impl Iterator<Item = &'static str> {
                ["test"].into_iter()
            }

            fn handle(
                &mut self,
                ctx: &MessageContext<MaelstromServerMessageSender>,
            ) -> Result<(), ErrorMessage> {
                ctx.reply(
                    "hello",
                    &TestResponseMessage1 {
                        foo: "there".to_owned(),
                    },
                )
            }
        }

        #[derive(Default)]
        struct TestHandler2;

        #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
        struct TestResponseMessage2 {
            bar: String,
        }

        impl MessageReceiver<MaelstromServerMessageSender> for TestHandler2 {
            fn new() -> Self {
                Self
            }

            fn get_handled_messages() -> impl Iterator<Item = &'static str> {
                ["test"].into_iter()
            }

            fn handle(
                &mut self,
                ctx: &MessageContext<MaelstromServerMessageSender>,
            ) -> Result<(), ErrorMessage> {
                ctx.reply(
                    "hi",
                    &TestResponseMessage2 {
                        bar: "wassup".to_owned(),
                    },
                )
            }
        }

        let mut handler = MaelstromServerMessageHandler::new();
        handler.register_handler::<TestHandler1>();
        handler.register_handler::<TestHandler2>();

        let msg = Message {
            src: None,
            dest: None,
            body: MessageBody {
                msg_id: Some(1),
                in_reply_to: None,
                content: MessageContent {
                    kind: "test".to_string(),
                    data: Default::default(),
                },
            },
        };

        let sender = MaelstromServerMessageSender::new();

        let ctx = MessageContext::new(Some(&msg), &[], &sender);
        let _ = handler.handle_message(&ctx);

        let reply1 = sender.pop();
        let reply2 = sender.pop();

        assert!(reply1.is_some_and(|reply1| {
            reply1.body.content.kind == "hello"
                && reply1.body.content.data.get("foo")
                    == Some(&serde_json::Value::String("there".to_owned()))
        }));
        assert!(reply2.is_some_and(|reply2| {
            reply2.body.content.kind == "hi"
                && reply2.body.content.data.get("bar")
                    == Some(&serde_json::Value::String("wassup".to_owned()))
        }));
    }

    #[test]
    fn test_multiple_handlers_failure() {
        #[derive(Default)]
        struct TestHandler1;

        impl MessageReceiver<MaelstromServerMessageSender> for TestHandler1 {
            fn new() -> Self {
                Self
            }

            fn get_handled_messages() -> impl Iterator<Item = &'static str> {
                ["test"].into_iter()
            }

            fn handle(
                &mut self,
                _ctx: &MessageContext<MaelstromServerMessageSender>,
            ) -> Result<(), ErrorMessage> {
                Err(ErrorMessage::new(ErrorKind::Crash, "hello there"))
            }
        }

        #[derive(Default)]
        struct TestHandler2;

        impl MessageReceiver<MaelstromServerMessageSender> for TestHandler2 {
            fn new() -> Self {
                Self
            }

            fn get_handled_messages() -> impl Iterator<Item = &'static str> {
                ["test"].into_iter()
            }

            fn handle(
                &mut self,
                _ctx: &MessageContext<MaelstromServerMessageSender>,
            ) -> Result<(), ErrorMessage> {
                Err(ErrorMessage::new(ErrorKind::Crash, "hi there"))
            }
        }

        let mut handler = MaelstromServerMessageHandler::new();
        handler.register_handler::<TestHandler1>();
        handler.register_handler::<TestHandler2>();

        let msg = Message {
            src: None,
            dest: None,
            body: MessageBody {
                msg_id: Some(1),
                in_reply_to: None,
                content: MessageContent {
                    kind: "test".to_string(),
                    data: Default::default(),
                },
            },
        };

        let sender = MaelstromServerMessageSender::new();
        let ctx = MessageContext::new(Some(&msg), &[], &sender);
        let res = handler.handle_message(&ctx);

        assert!(
            res.is_err_and(
                |x| x.code() == usize::from(ErrorKind::Crash) && x.text() == "hello there"
            )
        );
    }
}
