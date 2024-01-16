use crate::{
    bus::{
        DynamicMap, Message, MessageBody, MessageContent, MessageContext, MessageReceiver,
        MessageSender,
    },
    errors::{ErrorKind, ErrorMessage},
};
use anyhow::Result;
use serde::{Deserialize, Deserializer, Serialize};
use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct InitMessage {
    node_id: String,
    node_ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct InitOkMessage;

pub struct MaelstromServer {
    handler: MaelstromServerMessageHandler,
    sender: MaelstromServerMessageSender,
    node: Option<MaelstromServerNode>,
}

type MessageHandleResult<'a> = (
    MessageContext<'a, MaelstromServerMessageSender>,
    Result<(), ErrorMessage>,
);

impl MaelstromServer {
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
        self.node.as_ref().and_then(|node| {
            msg.map(|msg| Message {
                src: node.node_id.clone(),
                ..msg
            })
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

struct MaelstromServerNode {
    node_id: Option<String>,
    node_ids: Vec<String>,
}

impl MaelstromServerNode {
    fn create(ctx: &MessageContext<MaelstromServerMessageSender>) -> Result<Self, ErrorMessage> {
        let init_msg = ctx.message_content::<InitMessage>()?;

        let node_id = Some(init_msg.node_id.to_string());
        let node_ids = init_msg.node_ids;

        ctx.broadcast("init_ok", &InitOkMessage)?;

        Ok(Self { node_id, node_ids })
    }
}

struct MaelstromServerMessageSender {
    outgoing_msgs: RefCell<VecDeque<Message>>,
}

impl MaelstromServerMessageSender {
    fn new() -> Self {
        Self {
            outgoing_msgs: RefCell::new(VecDeque::new()),
        }
    }

    fn pop(&self) -> Option<Message> {
        self.outgoing_msgs.borrow_mut().pop_front()
    }
}

impl MessageSender for MaelstromServerMessageSender {
    fn send(&self, kind: &str, data: DynamicMap, dest: Option<&str>, in_reply_to: Option<usize>) {
        let mut outgoing_msgs = self.outgoing_msgs.borrow_mut();
        let msg_id = Some(outgoing_msgs.len() + 1);

        outgoing_msgs.push_back(Message {
            src: None,
            dest: dest.map(|s| s.to_owned()),
            body: MessageBody {
                in_reply_to,
                msg_id,
                content: MessageContent {
                    kind: kind.to_string(),
                    data,
                },
            },
        })
    }
}

struct MaelstromServerMessageHandler {
    msg_handlers: HashMap<String, usize>,
    handlers: Vec<Box<dyn MessageReceiver<MaelstromServerMessageSender>>>,
}

impl MaelstromServerMessageHandler {
    fn new() -> Self {
        Self {
            msg_handlers: HashMap::new(),
            handlers: Vec::new(),
        }
    }

    #[allow(private_bounds)]
    fn register_handler<T>(&mut self)
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

    fn handle_message(
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
