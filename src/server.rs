use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Message<T> {
    src: String,
    dest: String,
    body: MessageBody<T>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MessageBody<T> {
    msg_id: Option<usize>,
    in_reply_to: Option<usize>,
    #[serde(flatten)]
    content: MessageContent<T>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged, rename_all = "snake_case")]
pub enum MessageContent<T> {
    System(SystemMessage),
    Custom(T),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SystemMessage {
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk,
    Error {
        code: usize,
        text: String,
    },
}

pub trait MessageHandler {
    type Request: for<'a> Deserialize<'a>;
    type Response: Serialize;
    type Error: Into<anyhow::Error> + Sync + Send + 'static;

    fn init(
        &mut self,
        _node_id: &str,
        _node_ids: &[&str],
    ) -> core::result::Result<(), Self::Error> {
        Ok(())
    }

    fn handle(
        &mut self,
        msg: Self::Request,
    ) -> core::result::Result<Option<Self::Response>, Self::Error>;
}

pub struct MaelstromServer<T>
where
    T: MessageHandler,
{
    inner: T,
    last_msg_id: usize,
    node_id: Option<String>,
}

impl<T> MaelstromServer<T>
where
    T: MessageHandler,
{
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            last_msg_id: 1,
            node_id: None,
        }
    }

    pub fn create_error(&mut self, code: usize, text: &str) -> Message<T::Response> {
        self.new_reply(
            "",
            MessageContent::System(SystemMessage::Error {
                code,
                text: text.to_string(),
            }),
            None,
        )
    }

    pub fn handle(&mut self, msg: Message<T::Request>) -> Result<Option<Message<T::Response>>> {
        Ok(match msg.body.content {
            MessageContent::System(SystemMessage::Init { node_id, node_ids }) => {
                self.node_id = Some(node_id.clone());
                self.inner
                    .init(
                        &node_id,
                        &node_ids.iter().map(|s| s.as_str()).collect::<Vec<&str>>(),
                    )
                    .map_err(|err| err.into().context("failed to initialize message handler"))?;

                Some(self.new_reply(
                    &msg.src,
                    MessageContent::System(SystemMessage::InitOk),
                    msg.body.msg_id,
                ))
            }

            MessageContent::Custom(content) => {
                if self.node_id.is_none() {
                    return Err(anyhow!("node ID not set"));
                }

                let response = self.inner.handle(content).map_err(|err| {
                    err.into()
                        .context("downstream handler failed to handle message")
                })?;

                response
                    .map(|r| self.new_reply(&msg.src, MessageContent::Custom(r), msg.body.msg_id))
            }

            _ => None,
        })
    }

    fn new_reply(
        &mut self,
        to: &str,
        content: MessageContent<T::Response>,
        reply_to: Option<usize>,
    ) -> Message<T::Response> {
        self.last_msg_id += 1;
        Message {
            src: self.node_id.clone().unwrap_or("".to_string()),
            dest: to.to_string(),
            body: MessageBody {
                msg_id: Some(self.last_msg_id),
                in_reply_to: reply_to,
                content,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Context;

    use super::*;

    struct TestHandler;

    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    enum TestMessageContent {
        Hello,
        There,
    }

    impl MessageHandler for TestHandler {
        type Request = TestMessageContent;
        type Response = TestMessageContent;
        type Error = anyhow::Error;

        fn init(
            &mut self,
            _node_id: &str,
            _node_ids: &[&str],
        ) -> core::result::Result<(), Self::Error> {
            Ok(())
        }

        fn handle(
            &mut self,
            _msg: Self::Request,
        ) -> core::result::Result<Option<Self::Response>, Self::Error> {
            Ok(Some(Self::Response::There))
        }
    }

    #[test]
    fn test_handle_init_responds_init_ok() {
        let mut server = MaelstromServer::new(TestHandler);

        let msg = Message {
            src: "src".to_string(),
            dest: "n1".to_string(),
            body: MessageBody {
                msg_id: Some(1),
                in_reply_to: None,
                content: MessageContent::System(SystemMessage::Init {
                    node_id: "n1".to_string(),
                    node_ids: ["n1", "n2"].iter().map(|s| s.to_string()).collect(),
                }),
            },
        };

        let response = server.handle(msg).unwrap().unwrap();

        assert_eq!(response.src, "n1");
        assert_eq!(response.dest, "src");
        assert_eq!(response.body.msg_id, Some(2));
        assert_eq!(response.body.in_reply_to, Some(1));
        assert_eq!(
            response.body.content,
            MessageContent::System(SystemMessage::InitOk)
        );
    }

    #[test]
    fn test_handle_custom_messages_response() {
        let mut server = MaelstromServer::new(TestHandler);

        server
            .handle(Message {
                src: "src".to_string(),
                dest: "n1".to_string(),
                body: MessageBody {
                    msg_id: Some(1),
                    in_reply_to: None,
                    content: MessageContent::System(SystemMessage::Init {
                        node_id: "n1".to_string(),
                        node_ids: ["n1", "n2"].iter().map(|s| s.to_string()).collect(),
                    }),
                },
            })
            .context("initializing node")
            .unwrap()
            .unwrap();

        let response = server
            .handle(Message {
                src: "src".to_string(),
                dest: "n1".to_string(),
                body: MessageBody {
                    msg_id: Some(3),
                    in_reply_to: None,
                    content: MessageContent::Custom(TestMessageContent::Hello),
                },
            })
            .unwrap()
            .unwrap();

        assert_eq!(response.src, "n1");
        assert_eq!(response.dest, "src");
        assert_eq!(response.body.msg_id, Some(3));
        assert_eq!(response.body.in_reply_to, Some(3));
        assert_eq!(
            response.body.content,
            MessageContent::Custom(TestMessageContent::There)
        );
    }
}
