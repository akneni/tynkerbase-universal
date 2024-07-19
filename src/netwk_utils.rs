use std::collections::HashMap;

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub node_id: String,
    pub name: String,
    pub email: String,
    pub addr: String,
}

impl Node {
    pub fn to_hashmap(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("node_id".to_string(), self.node_id.clone());
        map.insert("name".to_string(), self.name.clone());
        map.insert("email".to_string(), self.email.clone());
        map.insert("addr".to_string(), self.addr.clone());

        map
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct NodeDiags {
    pub node_id: String,
    pub name: String,
    pub manufacturer: Option<String>,
    pub cpu: Option<String>,
    pub cpu_arc: Option<String>,
    pub hardware_threads: Option<usize>,
    pub l1_cache_d: Option<String>,
    pub l1_cache_i: Option<String>,
    pub l2_cache: Option<String>,
    pub l3_cache: Option<String>,
    pub mem_total: Option<f64>,
    pub mem_free: Option<f64>,
}

impl NodeDiags {
    pub fn new(node_id: impl Into<String>, name: impl Into<String>) -> Self {
        NodeDiags {
            node_id: node_id.into(),
            name: name.into(),
            ..NodeDiags::default()
        }
    }
}