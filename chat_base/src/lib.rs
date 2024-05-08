use serde::{Deserialize, Serialize};

pub const PORT: u16 = 12345;

#[derive(Serialize, Deserialize, Debug)]
pub enum MessageType {
    JOIN(User),
    MESSAGE(User, String),
    LEAVE(User)
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct User {
    pub name: String,
}


