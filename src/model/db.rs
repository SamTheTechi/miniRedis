use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
    time::Instant,
};
use tokio::sync::RwLock;

#[allow(dead_code)]
#[derive(Debug)]
pub enum Value {
    String(Vec<u8>),
    List(VecDeque<Vec<u8>>),
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Entry {
    pub value: Value,
    pub expires_at: Option<Instant>,
}

pub type DB = Arc<RwLock<HashMap<String, Entry>>>;

impl Value {
    pub fn to_resp_bytes(&self) -> Vec<u8> {
        match self {
            Value::String(bytes) => {
                let mut resp = Vec::new();
                resp.extend_from_slice(b"$");
                resp.extend_from_slice(bytes.len().to_string().as_bytes());
                resp.extend_from_slice(b"\r\n");
                resp.extend_from_slice(bytes);
                resp.extend_from_slice(b"\r\n");
                resp
            }

            Value::List(_) => b"*0\r\n".to_vec(),
        }
    }
}
