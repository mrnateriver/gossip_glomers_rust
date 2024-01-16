use serde::{Deserialize, Serialize};

use crate::protocol::{ErrorMessage, MessageContext, MessageReceiver, MessageSender};

pub struct EchoMessageHandler;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EchoMessageContent {
    echo: String,
}

impl<S> MessageReceiver<S> for EchoMessageHandler
where
    S: MessageSender,
{
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

    fn handle(&mut self, ctx: &MessageContext<S>) -> Result<(), ErrorMessage> {
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
    use crate::protocol::{DynamicMap, Message, MessageBody, MessageContent};
    use serde_json::{Map, Value};
    use std::cell::RefCell;

    struct TestMessageSender {
        outgoing_msgs: RefCell<Vec<Message>>,
    }

    impl TestMessageSender {
        fn new() -> Self {
            TestMessageSender {
                outgoing_msgs: RefCell::new(vec![]),
            }
        }
    }

    impl MessageSender for TestMessageSender {
        fn send(
            &self,
            kind: &str,
            data: DynamicMap,
            dest: Option<&str>,
            in_reply_to: Option<usize>,
        ) {
            self.outgoing_msgs.borrow_mut().push(Message {
                src: Some("n1".to_string()),
                dest: dest.map(|s| s.to_owned()),
                body: MessageBody {
                    in_reply_to,
                    msg_id: Some(123),
                    content: MessageContent {
                        kind: kind.to_string(),
                        data,
                    },
                },
            })
        }
    }

    #[test]
    fn test_echo_message_handler() {
        let mut handler = EchoMessageHandler;
        let sender = TestMessageSender::new();

        let mut echo_data = Map::new();
        echo_data.insert("echo".to_string(), Value::String("hello".to_string()));

        let res = handler.handle(&MessageContext::new(
            Some(&Message {
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
            }),
            &[],
            &sender,
        ));

        if let Err(err) = &res {
            eprintln!("{}", err);
        }

        assert!(res.is_ok());

        let borrow = sender.outgoing_msgs.borrow();
        let response = borrow.last().unwrap();

        assert_eq!(response.body.content.data, echo_data);
        assert_eq!(response.dest, Some("n2".to_string()));
        assert_eq!(response.body.in_reply_to, Some(123));
    }
}
