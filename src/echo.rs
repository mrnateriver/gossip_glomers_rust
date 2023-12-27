use serde::{Deserialize, Serialize};

use crate::server::MessageHandler;

pub struct EchoMessageHandler;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EchoMessageContent {
    Echo { echo: String },
    EchoOk { echo: String },
}

impl MessageHandler for EchoMessageHandler {
    type Request = EchoMessageContent;
    type Response = EchoMessageContent;
    type Error = std::io::Error;

    fn handle(&mut self, msg: Self::Request) -> Result<Option<Self::Response>, Self::Error> {
        Ok(match msg {
            EchoMessageContent::Echo { echo } => Some(EchoMessageContent::EchoOk { echo }),
            _ => None,
        })
    }
}

impl EchoMessageHandler {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_echo_message_handler() {
        let mut handler = EchoMessageHandler;
        let msg = EchoMessageContent::Echo {
            echo: "hello".to_string(),
        };
        let response = handler.handle(msg).unwrap();
        assert_eq!(
            response,
            Some(EchoMessageContent::EchoOk {
                echo: "hello".to_string()
            })
        );
    }
}
