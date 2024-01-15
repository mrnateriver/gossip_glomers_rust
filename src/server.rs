use crate::{
    bus::{
        DynamicMap, Message, MessageBody, MessageContent, MessageContext, MessageId,
        MessageReceiver, MessageSender, MessageType, NodeId,
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
    msg_handlers: HashMap<MessageType, usize>,
    handlers: Vec<Box<dyn MessageReceiver<MaelstromServerMessageSender>>>,
    sender: MaelstromServerMessageSender,
    node: Option<MaelstromServerNode>,
}

impl MaelstromServer {
    pub fn new() -> Self {
        Self {
            msg_handlers: HashMap::new(),
            handlers: Vec::new(),
            sender: MaelstromServerMessageSender::new(),
            node: None,
        }
    }

    pub fn register_handler<T>(&mut self)
    where
        T: MessageReceiver<MaelstromServerMessageSender> + 'static,
    {
        let handle_idx = self.handlers.len();
        self.handlers.push(Box::new(T::new()));

        let msg_types = T::get_handled_messages();
        for msg_type in msg_types {
            self.msg_handlers.insert(msg_type, handle_idx);
        }
    }

    pub fn input<'de, D>(&mut self, deserializer: D)
    where
        D: Deserializer<'de>,
    {
        match Message::deserialize(deserializer) {
            Err(err) => {
                let ctx = MessageContext::empty(&self.sender);
                let error = ErrorMessage::new(ErrorKind::MalformedRequest, &format!("{}", err));
                ctx.error(&error);
            }

            Ok(ref msg) => {
                if let Err(ref error) = self.handle(msg) {
                    if let Some(ref node) = self.node {
                        let ctx = MessageContext::new(Some(msg), &node.node_ids, &self.sender);
                        ctx.error(error);
                    }
                }
            }
        };
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

    fn handle(&mut self, msg: &Message) -> Result<(), ErrorMessage> {
        match msg.kind() {
            "init" => self.handle_init(msg),
            kind => self.handle_arbitrary(kind, msg),
        }
    }

    fn handle_init(&mut self, msg: &Message) -> Result<(), ErrorMessage> {
        let ctx = &MessageContext::empty(&self.sender).with_message(msg);
        self.node = Some(MaelstromServerNode::create(ctx)?);
        Ok(())
    }

    fn handle_arbitrary(&mut self, kind: &str, msg: &Message) -> Result<(), ErrorMessage> {
        if let Some(ref node) = self.node {
            let ctx = MessageContext::new(Some(msg), &node.node_ids, &self.sender);

            if let Some(&handler_idx) = self.msg_handlers.get(kind) {
                self.handlers[handler_idx].handle(&ctx)
            } else {
                Err(ErrorMessage::new(
                    ErrorKind::NotSupported,
                    &format!("message type {kind} not supported"),
                ))
            }
        } else {
            Err(ErrorMessage::new(
                ErrorKind::PreconditionFailed,
                &format!("node is not initialized before handling message type {kind}"),
            ))
        }
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
    fn send(
        &self,
        kind: &str,
        data: DynamicMap,
        dest: Option<NodeId>,
        in_reply_to: Option<MessageId>,
    ) {
        let mut outgoing_msgs = self.outgoing_msgs.borrow_mut();
        let msg_id = Some(outgoing_msgs.len() + 1);

        outgoing_msgs.push_back(Message {
            src: None,
            dest,
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
