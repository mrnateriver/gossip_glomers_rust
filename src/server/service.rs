use serde::{Deserialize, Deserializer};

use crate::protocol::{ErrorKind, ErrorMessage, Message, MessageContext, MessageHandler};

use super::{handler::MaelstromServerMessageHandler, node::MaelstromServerNode, InitMessage};

pub struct MaelstromService {
    handler: MaelstromServerMessageHandler,
    node: Option<MaelstromServerNode>,
}

impl MaelstromService {
    pub fn new() -> Self {
        Self {
            handler: MaelstromServerMessageHandler::new(),
            node: None,
        }
    }

    #[allow(private_bounds)]
    pub fn register_handler<T>(&mut self)
    where
        T: MessageHandler + 'static,
    {
        self.handler.register_handler::<T>()
    }

    pub fn input<'de, D>(&mut self, deserializer: D) -> impl Iterator<Item = Message>
    where
        D: Deserializer<'de>,
    {
        let message = Message::deserialize(deserializer);

        let ctx = message.map(|msg| MessageContext::new(Some(msg)));

        let res = ctx
            .as_ref()
            .map_err(|err| ErrorMessage::new(ErrorKind::MalformedRequest, &format!("{}", err)))
            .and_then(|ctx| self.handle(ctx));

        let ctx = ctx.unwrap_or_default();

        if let Err(error) = res {
            let _ = ctx.error(&error);
        }

        ctx.into_output_iter()
    }

    fn handle(&mut self, ctx: &MessageContext) -> Result<(), ErrorMessage> {
        match ctx.message_kind() {
            "init" => self.handle_init(ctx),
            _ => self.handler.handle_message(ctx),
        }
    }

    fn handle_init(&mut self, ctx: &MessageContext) -> Result<(), ErrorMessage> {
        MaelstromServerNode::create(ctx).map(|node| {
            self.node = Some(node);
        })?;

        let init_msg = ctx.message_content::<InitMessage>().unwrap();
        self.handler.handle_init(&init_msg, ctx)
    }
}
