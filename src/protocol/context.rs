use serde::{de::DeserializeOwned, Serialize};

use super::{
    serialization::{deserialize_message_content, serialize_message_content},
    DynamicMap, ErrorKind, ErrorMessage, Message,
};

pub struct MessageContext<'a, S>
where
    S: MessageSender,
{
    msg: Option<&'a Message>,
    sender: &'a S,
    node_ids: &'a [String],
}

impl<'a, S> MessageContext<'a, S>
where
    S: MessageSender,
{
    pub fn empty(sender: &'a S) -> Self {
        Self {
            msg: None,
            node_ids: &[],
            sender,
        }
    }

    pub fn new(msg: Option<&'a Message>, node_ids: &'a [String], sender: &'a S) -> Self {
        Self {
            msg,
            node_ids,
            sender,
        }
    }

    pub fn with_message(self, msg: &'a Message) -> Self {
        Self {
            msg: Some(msg),
            ..self
        }
    }

    pub fn with_node_ids(self, node_ids: &'a [String]) -> Self {
        Self { node_ids, ..self }
    }

    pub fn available_node_ids(&'a self) -> &'a [String] {
        self.node_ids
    }

    pub fn message_dest(&'a self) -> Option<&str> {
        self.msg
            .and_then(|msg| msg.dest.as_ref().map(|s| s.as_ref()))
    }

    pub fn message_src(&'a self) -> Option<&str> {
        self.msg
            .and_then(|msg| msg.src.as_ref().map(|s| s.as_ref()))
    }

    pub fn message_kind(&'a self) -> &str {
        self.msg.map(|msg| msg.kind()).unwrap_or_default()
    }

    pub fn message_id(&'a self) -> Option<usize> {
        self.msg.and_then(|msg| msg.body.msg_id)
    }

    pub fn message_in_reply_to(&'a self) -> Option<usize> {
        self.msg.and_then(|msg| msg.body.in_reply_to)
    }

    pub fn message_content<T>(&'a self) -> Result<T, ErrorMessage>
    where
        T: DeserializeOwned,
    {
        if let Some(msg) = self.msg {
            deserialize_message_content(msg)
        } else {
            Err(ErrorMessage::new(ErrorKind::Crash, "message not available"))
        }
    }

    pub fn reply<T>(&'a self, kind: &str, data: &T) -> Result<(), ErrorMessage>
    where
        T: Serialize,
    {
        self.send(
            kind,
            data,
            self.msg
                .and_then(|msg| msg.src.as_ref().map(|s| s.as_ref())),
            self.msg.and_then(|msg| msg.body.msg_id),
        )
    }

    pub fn broadcast<T>(&'a self, kind: &str, data: &T) -> Result<(), ErrorMessage>
    where
        T: Serialize,
    {
        self.send(kind, data, None, None)
    }

    pub fn error(&'a self, error: &ErrorMessage) -> Result<(), ErrorMessage> {
        self.reply("error", error)
    }

    fn send<T>(
        &'a self,
        kind: &str,
        data: &T,
        dest: Option<&str>,
        in_reply_to: Option<usize>,
    ) -> Result<(), ErrorMessage>
    where
        T: Serialize,
    {
        self.sender
            .send(kind, serialize_message_content(data)?, dest, in_reply_to);
        Ok(())
    }
}

pub trait MessageSender {
    fn send(&self, kind: &str, data: DynamicMap, dest: Option<&str>, in_reply_to: Option<usize>);
}

pub trait MessageReceiver<S>
where
    S: MessageSender,
{
    fn new() -> Self
    where
        Self: Sized;

    fn get_handled_messages() -> impl Iterator<Item = &'static str>
    where
        Self: Sized;

    fn handle(&mut self, ctx: &MessageContext<S>) -> Result<(), ErrorMessage>;
}
