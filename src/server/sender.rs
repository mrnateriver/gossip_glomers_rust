use std::{cell::RefCell, collections::VecDeque};

use crate::protocol::{DynamicMap, Message, MessageBody, MessageContent, MessageSender};

pub struct MaelstromServerMessageSender {
    node_ids: Vec<String>,
    outgoing_msgs: RefCell<VecDeque<Message>>,
}

impl MaelstromServerMessageSender {
    pub fn new() -> Self {
        Self {
            node_ids: Vec::new(),
            outgoing_msgs: RefCell::new(VecDeque::new()),
        }
    }

    pub fn pop(&self) -> Option<Message> {
        self.outgoing_msgs.borrow_mut().pop_front()
    }

    pub fn set_node_ids(&mut self, node_ids: &[String]) {
        self.node_ids = Vec::from(node_ids);
    }
}

impl MessageSender for MaelstromServerMessageSender {
    fn send(&self, kind: &str, data: DynamicMap, dest: Option<&str>, in_reply_to: Option<usize>) {
        let mut outgoing_msgs = self.outgoing_msgs.borrow_mut();
        let msg_id = Some(outgoing_msgs.len() + 1);

        outgoing_msgs.push_back(Message {
            src: None,
            dest: dest.map(|s| s.to_owned()),
            body: MessageBody {
                in_reply_to,
                msg_id,
                content: MessageContent {
                    kind: kind.to_string(),
                    data,
                },
            },
        })
    }

    fn fan_out(&self, kind: &str, data: DynamicMap) {
        for node_id in self.node_ids.iter() {
            self.send(kind, data.clone(), Some(node_id), None);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fan_out() {
        let mut sender = MaelstromServerMessageSender::new();
        sender.set_node_ids(&["n1".to_owned(), "n2".to_owned()]);

        sender.fan_out("test", Default::default());

        let mut outgoing_msgs = sender.outgoing_msgs.borrow_mut();
        assert_eq!(outgoing_msgs.len(), 2);

        let msg = outgoing_msgs.pop_front().unwrap();
        assert_eq!(msg.dest, Some("n1".to_owned()));

        let msg = outgoing_msgs.pop_front().unwrap();
        assert_eq!(msg.dest, Some("n2".to_owned()));
    }
}
