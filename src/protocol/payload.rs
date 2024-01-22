use serde::{Deserialize, Serialize};

pub type DynamicMap = serde_json::Map<String, serde_json::Value>;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Message {
    pub src: Option<String>,
    pub dest: Option<String>,
    pub body: MessageBody,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct MessageBody {
    pub msg_id: Option<usize>,
    pub in_reply_to: Option<usize>,
    #[serde(flatten)]
    pub content: MessageContent,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct MessageContent {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(flatten)]
    pub data: DynamicMap,
}

impl Message {
    pub fn kind(&self) -> &str {
        self.body.content.kind.as_ref()
    }
}
