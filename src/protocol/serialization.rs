use serde::{de::DeserializeOwned, Serialize};

use super::{DynamicMap, ErrorKind, ErrorMessage, Message};

pub fn deserialize_message_content<T>(msg: &Message) -> Result<T, ErrorMessage>
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
    let value = serde_json::to_value(data);
    match value {
        Ok(serde_json::Value::Null) => Ok(DynamicMap::new()),
        Ok(serde_json::Value::Object(map)) => Ok(map),
        _ => Err(ErrorMessage::new(
            ErrorKind::Crash,
            "message content must serialize to an object",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{MessageBody, MessageContent};
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
