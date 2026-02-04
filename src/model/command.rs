use super::Entry;

#[derive(Debug)]
#[allow(dead_code)]
pub enum Command {
    PING,
    SET { key: String, value: Entry },
    SETEX { key: String, value: Entry },
    PSETEX { key: String, value: Entry },
    GET { key: String },
    DEL { keys: Vec<String> },
    EXISTS { keys: Vec<String> },
    EXPIRE { key: String, sec: u64 },
    TTL { key: String },
    PTTL { key: String },
    LPUSH { key: String, value: Entry },
    RPUSH { key: String, value: Entry },
    LPOP { key: String },
    RPOP { key: String },
}
