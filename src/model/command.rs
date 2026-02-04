use super::Entry;

#[derive(Debug)]
#[allow(dead_code)]
#[rustfmt::skip]
pub enum Command {
    PING,
    SET { key: String, value: Vec<u8> },
    SETEX { key: String, value:  Vec<u8>, seconds: u64 },
    PSETEX { key: String, value: Vec<u8>, seconds: u64 },
    GET { key: String },
    DEL { keys: Vec<String> },
    EXISTS { keys: Vec<String> },
    EXPIRE { key: String, seconds: u64 },
    TTL { key: String },
    PTTL { key: String },
    LPUSH { key: String, values: Vec<Vec<u8>> },
    RPUSH { key: String, values: Vec<Vec<u8>> },
    LPOP { key: String },
    RPOP { key: String },
}
