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
}

impl<'a, S> MessageContext<'a, S>
where
    S: MessageSender,
{
    pub fn empty(sender: &'a S) -> Self {
        Self { msg: None, sender }
    }

    pub fn new(msg: Option<&'a Message>, sender: &'a S) -> Self {
        Self { msg, sender }
    }

    pub fn with_message(self, msg: &'a Message) -> Self {
        Self {
            msg: Some(msg),
            ..self
        }
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

    pub fn error(&'a self, error: &ErrorMessage) -> Result<(), ErrorMessage> {
        self.reply("error", error)
    }

    pub fn broadcast<T>(&'a self, kind: &str, data: &T) -> Result<(), ErrorMessage>
    where
        T: Serialize,
    {
        self.send(kind, data, None, None)
    }

    pub fn fan_out<T>(&'a self, kind: &str, data: &T) -> Result<(), ErrorMessage>
    where
        T: Serialize,
    {
        self.sender.fan_out(kind, serialize_message_content(data)?);
        Ok(())
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

    fn fan_out(&self, kind: &str, data: DynamicMap);
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

    fn init(
        &mut self,
        node_id: &str,
        node_ids: &[String],
        _: &MessageContext<S>,
    ) -> Result<(), ErrorMessage> {
        Ok(())
    }

    fn handle(&mut self, ctx: &MessageContext<S>) -> Result<(), ErrorMessage>;
}
