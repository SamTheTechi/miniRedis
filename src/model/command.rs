use super::Entry;

#[derive(Debug)]
pub enum Command {
    PING,
    SET { key: String, value: Entry },
    GET { key: String },
    DEL { keys: Vec<String> },
    EXISTS { keys: Vec<String> },
    EXPIRE { key: String, sec: u64 },
    TTL { key: String },
}
