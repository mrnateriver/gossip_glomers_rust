use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::errors::{ErrorKind, ErrorMessage};

pub use payload::*;
pub use traits::*;

pub mod payload {
    use super::*;

    pub type NodeId = String;
    pub type MessageId = usize;
    pub type MessageType = String;
    pub type DynamicMap = serde_json::Map<String, serde_json::Value>;

    #[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
    pub struct Message {
        pub src: Option<NodeId>,
        pub dest: Option<NodeId>,
        pub body: MessageBody,
    }

    #[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
    pub struct MessageBody {
        pub msg_id: Option<MessageId>,
        pub in_reply_to: Option<MessageId>,
        #[serde(flatten)]
        pub content: MessageContent,
    }

    #[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
    pub struct MessageContent {
        #[serde(rename = "type")]
        pub kind: MessageType,
        #[serde(flatten)]
        pub data: DynamicMap,
    }

    impl Message {
        pub fn kind(&self) -> &str {
            self.body.content.kind.as_ref()
        }
    }
}

pub mod traits {
    use super::*;

    pub struct MessageContext<'a, S>
    where
        S: MessageSender,
    {
        msg: Option<&'a Message>,
        sender: &'a S,
        node_ids: &'a [NodeId],
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

        pub fn new(msg: Option<&'a Message>, node_ids: &'a [NodeId], sender: &'a S) -> Self {
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

        pub fn with_node_ids(self, node_ids: &'a [NodeId]) -> Self {
            Self { node_ids, ..self }
        }

        pub fn available_node_ids(&'a self) -> &'a [NodeId] {
            self.node_ids
        }

        pub fn message_dest(&'a self) -> Option<&NodeId> {
            self.msg.and_then(|msg| msg.dest.as_ref())
        }

        pub fn message_src(&'a self) -> Option<&NodeId> {
            self.msg.and_then(|msg| msg.src.as_ref())
        }

        pub fn message_kind(&'a self) -> &str {
            self.msg.map(|msg| msg.kind().as_ref()).unwrap_or_default()
        }

        pub fn message_id(&'a self) -> Option<MessageId> {
            self.msg.and_then(|msg| msg.body.msg_id)
        }

        pub fn message_in_reply_to(&'a self) -> Option<MessageId> {
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
                self.msg.and_then(|msg| msg.src.clone()),
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
            dest: Option<NodeId>,
            in_reply_to: Option<MessageId>,
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
        fn send(
            &self,
            kind: &str,
            data: DynamicMap,
            dest: Option<NodeId>,
            in_reply_to: Option<MessageId>,
        );
    }

    pub trait MessageReceiver<S>
    where
        S: MessageSender,
    {
        fn new() -> Self
        where
            Self: Sized;

        fn get_handled_messages() -> impl Iterator<Item = MessageType>
        where
            Self: Sized;

        fn handle(&mut self, ctx: &MessageContext<S>) -> Result<(), ErrorMessage>;
    }
}

fn deserialize_message_content<T>(msg: &Message) -> Result<T, ErrorMessage>
where
    T: DeserializeOwned,
{
    T::deserialize(serde_json::Value::Object(msg.body.content.data.clone())).map_err(|err| {
        ErrorMessage::new(
            ErrorKind::MalformedRequest,
            &format!("failed to deserialize message `{}`", msg.body.content.kind),
        )
        .with_source(err)
    })
}

pub fn serialize_message_content<T>(data: &T) -> Result<DynamicMap, ErrorMessage>
where
    T: Serialize,
{
    if let Ok(serde_json::Value::Object(map)) = serde_json::to_value(data) {
        Ok(map)
    } else {
        Err(ErrorMessage::new(
            ErrorKind::Crash,
            "message content must serialize to an object",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus::{Message, MessageBody, MessageContent};
    use serde_json::{Map, Value};

    #[test]
    fn test_deserialize_message_content() {
        let msg = Message {
            src: None,
            dest: None,
            body: MessageBody {
                in_reply_to: None,
                msg_id: None,
                content: MessageContent {
                    kind: "test".to_string(),
                    data: {
                        let mut map = Map::new();
                        map.insert("foo".to_string(), Value::String("bar".to_string()));
                        map
                    },
                },
            },
        };

        let data: Map<String, Value> = deserialize_message_content(&msg).unwrap();
        assert_eq!(data.get("foo").unwrap(), &Value::String("bar".to_string()));
    }

    #[test]
    fn test_serialize_message_content() {
        let data = {
            let mut map = Map::new();
            map.insert("foo".to_string(), Value::String("bar".to_string()));
            map
        };

        let serialized = serialize_message_content(&data).unwrap();
        assert_eq!(
            serialized.get("foo").unwrap(),
            &Value::String("bar".to_string())
        );
    }
}