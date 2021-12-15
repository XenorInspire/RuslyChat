use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct User {
    pub email: String,
    pub username: String,
}