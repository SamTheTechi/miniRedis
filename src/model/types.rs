use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

pub type DB = Arc<RwLock<HashMap<String, RESP>>>;

#[derive(Debug)]
#[allow(dead_code)]
pub enum Command {
    PING,
    SET { key: String, value: RESP },
    GET { key: String },
    DEL { keys: Vec<String> },
    EXISTS { keys: Vec<String> },
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum RESP {
    SimpleStrings(String),
    SimpleErrors(String),
    Integers(i64),
    BulkStrings(Option<Vec<u8>>),
    Arrays(Vec<RESP>),
}

impl RESP {
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            RESP::SimpleStrings(s) => format!("+{}\r\n", s).into_bytes(),
            RESP::SimpleErrors(e) => format!("-{}\r\n", e).into_bytes(),
            RESP::Integers(i) => format!(":{}\r\n", i).into_bytes(),
            RESP::BulkStrings(Some(b)) => {
                let mut out = Vec::new();
                out.extend_from_slice(format!("${}\r\n", b.len()).as_bytes());
                out.extend_from_slice(b);
                out.extend_from_slice(b"\r\n");
                out
            }
            RESP::BulkStrings(None) => b"$-1\r\n".to_vec(),
            RESP::Arrays(items) => {
                let mut out = Vec::new();
                out.extend_from_slice(format!("*{}\r\n", items.len()).as_bytes());
                for item in items {
                    out.extend_from_slice(&item.to_bytes());
                }
                out
            }
        }
    }
}
