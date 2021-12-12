use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Message {
    pub content: String,
    pub date: String
}