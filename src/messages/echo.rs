use serde::{Deserialize, Serialize};

use crate::protocol::{ErrorMessage, MessageContext, MessageHandler};

pub struct EchoMessageHandler;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EchoMessageContent {
    echo: String,
}

impl MessageHandler for EchoMessageHandler {
    fn new() -> Self
    where
        Self: Sized,
    {
        EchoMessageHandler
    }

    fn get_handled_messages() -> impl Iterator<Item = &'static str>
    where
        Self: Sized,
    {
        ["echo"].into_iter()
    }

    fn handle(&mut self, ctx: &MessageContext) -> Result<(), ErrorMessage> {
        let msg = ctx.message_content::<EchoMessageContent>()?;

        ctx.reply(
            "echo",
            &EchoMessageContent {
                echo: msg.echo.clone(),
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{Message, MessageBody, MessageContent};
    use serde_json::{Map, Value};

    #[test]
    fn test_echo_message_handler() {
        let mut handler = EchoMessageHandler;

        let mut echo_data = Map::new();
        echo_data.insert("echo".to_string(), Value::String("hello".to_string()));

        let ctx = MessageContext::new(Some(Message {
            src: Some("n2".to_string()),
            dest: Some("n1".to_string()),
            body: MessageBody {
                msg_id: Some(123),
                in_reply_to: None,
                content: MessageContent {
                    kind: "echo".to_string(),
                    data: echo_data.clone(),
                },
            },
        }));
        let res = handler.handle(&ctx);

        if let Err(err) = &res {
            eprintln!("{}", err);
        }

        assert!(res.is_ok());

        let response = ctx.into_output_iter().next().unwrap();

        assert_eq!(response.body.content.data, echo_data);
        assert_eq!(response.dest, Some("n2".to_string()));
        assert_eq!(response.body.in_reply_to, Some(123));
    }
}
