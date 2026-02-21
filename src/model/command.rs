#[derive(Debug)]
#[rustfmt::skip]
pub enum Command {
    PING,
    QUIT,
    SET { key: String, value: Vec<u8> },
    SETEX { key: String, value:  Vec<u8>, seconds: u64 },
    PSETEX { key: String, value: Vec<u8>, seconds: u64 },
    GET { key: String },
    DEL { keys: Vec<String> },
    EXISTS { keys: Vec<String> },
    EXPIRE { key: String, seconds: u64 },
    PERSIST { key: String },
    TTL { key: String },
    PTTL { key: String },
    TYPE { key: String },
    INFO { section: Option<String> },
    HELLO { version: Option<u8> },
    COMMAND,
    ClientSetinfo,
    ConfigGet { pattern: String },
    ConfigSet { key: String, value: String },
    LPUSH { key: String, values: Vec<Vec<u8>> },
    RPUSH { key: String, values: Vec<Vec<u8>> },
    LPOP { key: String },
    RPOP { key: String },
}

pub struct CommandInfo<'a> {
    name: &'a str,
    arity: i64,
    flags: &'a [&'a str],
    first_key: i64,
    last_key: i64,
    key_step: i64,
}

impl<'a> CommandInfo<'a> {
    pub fn new(
        name: &'a str,
        arity: i64,
        flags: &'a [&'a str],
        first_key: i64,
        last_key: i64,
        key_step: i64,
    ) -> Self {
        Self {
            name,
            arity,
            flags,
            first_key,
            last_key,
            key_step,
        }
    }
    pub fn to_resp(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&crate::util::array_len(6));
        out.extend_from_slice(&crate::util::bulk_str(self.name));
        out.extend_from_slice(&crate::util::integer(self.arity));
        out.extend_from_slice(&flags_array(self.flags));
        out.extend_from_slice(&crate::util::integer(self.first_key));
        out.extend_from_slice(&crate::util::integer(self.last_key));
        out.extend_from_slice(&crate::util::integer(self.key_step));
        out
    }
}

fn flags_array(flags: &[&str]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&crate::util::array_len(flags.len()));
    for f in flags {
        out.extend_from_slice(&crate::util::bulk_str(f));
    }
    out
}
