use std::{
    cell::RefCell,
    collections::VecDeque,
    sync::atomic::{AtomicUsize, Ordering},
};

use serde::{de::DeserializeOwned, Serialize};

use super::{
    serialization::{deserialize_message_content, serialize_message_content},
    ErrorKind, ErrorMessage, Message, MessageBody, MessageContent,
};

static SHARED_MESSAGE_ID_COUNTER: AtomicUsize = AtomicUsize::new(1);

#[derive(Default)]
pub struct MessageContext {
    msg: Option<Message>,
    output: RefCell<VecDeque<Message>>,
}

impl MessageContext {
    pub fn new(msg: Option<Message>) -> Self {
        Self {
            msg,
            output: Default::default(),
        }
    }

    pub fn message_dest(&self) -> Option<&str> {
        self.msg
            .as_ref()
            .and_then(|msg| msg.dest.as_ref().map(|s| s.as_ref()))
    }

    pub fn message_src(&self) -> Option<&str> {
        self.msg
            .as_ref()
            .and_then(|msg| msg.src.as_ref().map(|s| s.as_ref()))
    }

    pub fn message_kind(&self) -> &str {
        self.msg.as_ref().map(|msg| msg.kind()).unwrap_or_default()
    }

    pub fn message_id(&self) -> Option<usize> {
        self.msg.as_ref().and_then(|msg| msg.body.msg_id)
    }

    pub fn message_in_reply_to(&self) -> Option<usize> {
        self.msg.as_ref().and_then(|msg| msg.body.in_reply_to)
    }

    pub fn message_content<T>(&self) -> Result<T, ErrorMessage>
    where
        T: DeserializeOwned,
    {
        if let Some(msg) = self.msg.as_ref() {
            deserialize_message_content(msg)
        } else {
            Err(ErrorMessage::new(ErrorKind::Crash, "message not available"))
        }
    }

    pub fn reply<T>(&self, kind: &str, data: &T) -> Result<(), ErrorMessage>
    where
        T: Serialize,
    {
        self.send(
            kind,
            data,
            self.msg
                .as_ref()
                .and_then(|msg| msg.src.as_ref().map(|s| s.as_ref())),
            self.msg.as_ref().and_then(|msg| msg.body.msg_id),
        )
    }

    pub fn error(&self, error: &ErrorMessage) -> Result<(), ErrorMessage> {
        self.reply("error", error)
    }

    pub fn broadcast<T>(&self, kind: &str, data: &T) -> Result<(), ErrorMessage>
    where
        T: Serialize,
    {
        self.send(kind, data, None, None)
    }

    pub fn into_output_iter(self) -> impl Iterator<Item = Message> {
        self.output.into_inner().into_iter()
    }

    fn send<T>(
        &self,
        kind: &str,
        data: &T,
        dest: Option<&str>,
        in_reply_to: Option<usize>,
    ) -> Result<(), ErrorMessage>
    where
        T: Serialize,
    {
        let msg = Message {
            src: self.message_dest().map(|s| s.to_owned()),
            dest: dest.map(|s| s.to_owned()),
            body: MessageBody {
                in_reply_to,
                msg_id: Some(SHARED_MESSAGE_ID_COUNTER.fetch_add(1, Ordering::Relaxed)),
                content: MessageContent {
                    kind: kind.to_string(),
                    data: serialize_message_content(data)?,
                },
            },
        };

        let mut outgoing_msgs = self.output.borrow_mut();
        outgoing_msgs.push_back(msg);

        Ok(())
    }
}
