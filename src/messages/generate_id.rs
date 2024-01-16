use serde::{Deserialize, Serialize};

use crate::protocol::{ErrorKind, ErrorMessage, MessageContext, MessageReceiver, MessageSender};

#[derive(Default)]
pub struct GenerateIdMessageHandler {
    max_id: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GenerateIdMessageContent;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GenerateIdOkMessageContent {
    id: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GetMaxIdMessageContent;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GetMaxIdOkMessageContent {
    id: usize,
}

// TODO: handle distributed failover

impl<S> MessageReceiver<S> for GenerateIdMessageHandler
where
    S: MessageSender,
{
    fn new() -> Self
    where
        Self: Sized,
    {
        Self::default()
    }

    fn get_handled_messages() -> impl Iterator<Item = &'static str>
    where
        Self: Sized,
    {
        ["generate_id", "get_max_id", "get_max_id_ok"].into_iter()
    }

    fn handle(&mut self, ctx: &MessageContext<S>) -> Result<(), ErrorMessage> {
        match ctx.message_kind() {
            "generate_id" => self.handle_generate_id(ctx),
            "get_max_id" => self.handle_get_max_id(ctx),
            "get_max_id_ok" => self.handle_get_max_id_ok(ctx),
            kind => Err(ErrorMessage::new(
                ErrorKind::NotSupported,
                &format!("message type {kind} not supported"),
            )),
        }
    }
}

impl GenerateIdMessageHandler {
    fn handle_generate_id<S: MessageSender>(
        &mut self,
        ctx: &MessageContext<S>,
    ) -> Result<(), ErrorMessage> {
        self.max_id += 1;
        ctx.reply(
            "generate_id_ok",
            &GenerateIdOkMessageContent { id: self.max_id },
        )
    }

    fn handle_get_max_id<S: MessageSender>(
        &mut self,
        ctx: &MessageContext<S>,
    ) -> Result<(), ErrorMessage> {
        ctx.reply(
            "get_max_id_ok",
            &GetMaxIdOkMessageContent { id: self.max_id },
        )
    }

    fn handle_get_max_id_ok<S: MessageSender>(
        &mut self,
        ctx: &MessageContext<S>,
    ) -> Result<(), ErrorMessage> {
        let msg = ctx.message_content::<GetMaxIdOkMessageContent>()?;

        self.max_id = msg.id;

        Ok(())
    }
}
