use serde::{Deserialize, Deserializer};

use crate::protocol::{ErrorKind, ErrorMessage, Message, MessageContext, MessageReceiver};

use super::{
    handler::MaelstromServerMessageHandler, node::MaelstromServerNode,
    sender::MaelstromServerMessageSender,
};

pub struct MaelstromService {
    handler: MaelstromServerMessageHandler,
    sender: MaelstromServerMessageSender,
    node: Option<MaelstromServerNode>,
}

type MessageHandleResult<'a> = (
    MessageContext<'a, MaelstromServerMessageSender>,
    Result<(), ErrorMessage>,
);

impl MaelstromService {
    pub fn new() -> Self {
        Self {
            handler: MaelstromServerMessageHandler::new(),
            sender: MaelstromServerMessageSender::new(),
            node: None,
        }
    }

    #[allow(private_bounds)]
    pub fn register_handler<T>(&mut self)
    where
        T: MessageReceiver<MaelstromServerMessageSender> + 'static,
    {
        self.handler.register_handler::<T>()
    }

    pub fn output(&mut self) -> Option<Message> {
        let msg = self.sender.pop();
        msg.map(|msg| Message {
            src: self.node.as_ref().and_then(|node| node.node_id.clone()),
            ..msg
        })
    }

    pub fn input<'de, D>(&mut self, deserializer: D)
    where
        D: Deserializer<'de>,
    {
        let message = Message::deserialize(deserializer);
        let (ctx, res) = match message {
            Err(err) => (
                MessageContext::empty(&self.sender),
                Err(ErrorMessage::new(
                    ErrorKind::MalformedRequest,
                    &format!("{}", err),
                )),
            ),

            Ok(ref msg) => self.handle(msg),
        };

        if let Err(error) = res {
            let _ = ctx.error(&error);
        }
    }

    fn handle<'a>(&'a mut self, msg: &'a Message) -> MessageHandleResult {
        let mut ctx = MessageContext::empty(&self.sender).with_message(msg);

        let res = match ctx.message_kind() {
            "init" => {
                let res = MaelstromServerNode::create(&ctx);
                res.map(|node| {
                    self.node = Some(node);
                })
            }

            _ => {
                if let Some(ref node) = self.node {
                    ctx = ctx.with_node_ids(&node.node_ids);
                    self.handler.handle_message(&ctx)
                } else {
                    Err(ErrorMessage::new(
                        ErrorKind::PreconditionFailed,
                        &format!(
                            "node is not initialized before handling message type {}",
                            ctx.message_kind()
                        ),
                    ))
                }
            }
        };

        (ctx, res)
    }
}
