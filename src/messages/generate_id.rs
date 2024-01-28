use serde::{Deserialize, Serialize};
use uuid::{
    timestamp::{self, context::Context},
    Uuid,
};

use crate::{
    protocol::{ErrorKind, ErrorMessage, MessageContext, MessageReceiver, MessageSender},
    server::InitMessage,
};

pub struct GenerateIdMessageHandler {
    node_id: Option<[u8; 6]>,
    ctx: Context,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GenerateIdMessageContent;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GenerateIdOkMessageContent {
    id: String,
}

impl<S> MessageReceiver<S> for GenerateIdMessageHandler
where
    S: MessageSender,
{
    fn new() -> Self
    where
        Self: Sized,
    {
        Self {
            node_id: None,
            ctx: Context::new_random(),
        }
    }

    fn get_handled_messages() -> impl Iterator<Item = &'static str>
    where
        Self: Sized,
    {
        ["generate", "init"].into_iter()
    }

    fn handle(&mut self, ctx: &MessageContext<S>) -> Result<(), ErrorMessage> {
        match ctx.message_kind() {
            "init" => self.handle_init(ctx),
            "generate" => self.handle_generate_id(ctx),
            kind => Err(ErrorMessage::new(
                ErrorKind::NotSupported,
                &format!("message type {kind} not supported"),
            )),
        }
    }
}

impl GenerateIdMessageHandler {
    fn handle_init<S: MessageSender>(
        &mut self,
        ctx: &MessageContext<S>,
    ) -> Result<(), ErrorMessage> {
        let init_msg = ctx.message_content::<InitMessage>()?;
        let node_id = init_msg.node_id.to_string();

        let digits = node_id.chars().skip(1).collect::<String>(); // Skip the "n" prefix

        let node_id = digits.parse::<usize>().map_err(|err| {
            ErrorMessage::new(
                ErrorKind::MalformedRequest,
                &format!("failed to parse node id `{}`", node_id),
            )
            .with_source(err)
        })?;

        let node_id_bytes = node_id.to_le_bytes()[0..6].to_owned();
        self.node_id = node_id_bytes.try_into().ok();

        Ok(())
    }

    fn handle_generate_id<S: MessageSender>(
        &mut self,
        ctx: &MessageContext<S>,
    ) -> Result<(), ErrorMessage> {
        if let Some(ref node_id) = self.node_id {
            let ts = timestamp::Timestamp::now(&self.ctx);
            let uuid = Uuid::new_v6(ts, node_id).to_string();

            ctx.reply("generate_ok", &GenerateIdOkMessageContent { id: uuid })
        } else {
            Err(ErrorMessage::new(
                ErrorKind::TemporarilyUnavailable,
                "node not initialized",
            ))
        }
    }
}
