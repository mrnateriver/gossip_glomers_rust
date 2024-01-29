use super::{ErrorMessage, MessageContext};

pub trait MessageHandler {
    fn new() -> Self
    where
        Self: Sized;

    fn get_handled_messages() -> impl Iterator<Item = &'static str>
    where
        Self: Sized;

    fn init(
        &mut self,
        _node_id: &str,
        _node_ids: &[String],
        _ctx: &MessageContext,
    ) -> Result<(), ErrorMessage> {
        Ok(())
    }

    fn handle(&mut self, ctx: &MessageContext) -> Result<(), ErrorMessage>;
}
