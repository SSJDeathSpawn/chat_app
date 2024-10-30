use serde::{Deserialize, Serialize};

pub const PORT: u16 = 12345;

#[derive(Serialize, Deserialize, Debug)]
pub enum MessageType {
    JOIN(User),
    MESSAGE(User, String),
    LEAVE(User)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub user: User,
    pub text: String
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerResponse {
    Ok,
    Err,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerInfo {
    Messaged(Message),
    Left(User),
    Join(User)
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct User {
    pub name: String,
}

