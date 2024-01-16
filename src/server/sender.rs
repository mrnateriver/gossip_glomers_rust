use std::{cell::RefCell, collections::VecDeque};

use crate::protocol::{DynamicMap, Message, MessageBody, MessageContent, MessageSender};

pub struct MaelstromServerMessageSender {
    outgoing_msgs: RefCell<VecDeque<Message>>,
}

impl MaelstromServerMessageSender {
    pub fn new() -> Self {
        Self {
            outgoing_msgs: RefCell::new(VecDeque::new()),
        }
    }

    pub fn pop(&self) -> Option<Message> {
        self.outgoing_msgs.borrow_mut().pop_front()
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
}
