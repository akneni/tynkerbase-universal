use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PubAddr {
    pub node_id: String,
    pub email: String,
    pub addr: String,
}