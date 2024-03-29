use crate::protocol::{ErrorMessage, MessageContext};

use super::system_messages::{InitMessage, InitOkMessage};

pub struct MaelstromServerNode {
    pub node_id: Option<String>,
    pub node_ids: Vec<String>,
}

impl MaelstromServerNode {
    pub fn create(ctx: &MessageContext) -> Result<Self, ErrorMessage> {
        let init_msg = ctx.message_content::<InitMessage>()?;

        let node_id = Some(init_msg.node_id.to_string());
        let node_ids = init_msg.node_ids;

        ctx.reply("init_ok", &InitOkMessage)?;

        Ok(Self { node_id, node_ids })
    }
}
