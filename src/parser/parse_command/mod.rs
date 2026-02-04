use crate::{
    model::{Command, Entry, RESP, Value},
    util::{bulk_to_string, expect_bulk},
};
use anyhow::{Ok, Result};
use std::time::{Duration, Instant};

pub fn parse_command(items: Vec<RESP>) -> Result<Command> {
    if items.is_empty() {
        return Err(anyhow::anyhow!("empty command"));
    }

    let cmd = match &items[0] {
        RESP::BulkStrings(Some(b)) => {
            let Some(bs) = bulk_to_string(b) else {
                return Err(anyhow::anyhow!("empty command"));
            };
            bs.to_uppercase()
        }
        _ => {
            return Err(anyhow::anyhow!("empty command"));
        }
    };

    match cmd.as_str() {
        "PING" => Ok(Command::PING),
        "GET" => {
            if items.len() != 2 {
                return Err(anyhow::anyhow!(
                    "wrong number of arguments for 'get' command"
                ));
            }

            let key = expect_bulk(&items, 1, "key")?;

            Ok(Command::GET { key })
        }
        "SET" => {
            if items.len() != 3 {
                return Err(anyhow::anyhow!(
                    "wrong number of arguments for 'set' command"
                ));
            }

            let key = expect_bulk(&items, 1, "key")?;
            let value = expect_bulk(&items, 3, "value")?;

            Ok(Command::SET {
                key,
                value: Entry {
                    value: Value::String(value.into_bytes()),
                    expires_at: None,
                },
            })
        }
        "SETEX" => {
            if items.len() != 4 {
                return Err(anyhow::anyhow!(
                    "wrong number of arguments for 'setex' command"
                ));
            }

            let key = expect_bulk(&items, 1, "key")?;
            let sec_str = expect_bulk(&items, 2, "time")?;
            let value = expect_bulk(&items, 3, "value")?;

            let sec: u64 = sec_str.parse().unwrap();

            Ok(Command::SET {
                key,
                value: Entry {
                    value: Value::String(value.into_bytes()),
                    expires_at: Some(Instant::now() + Duration::from_secs(sec)),
                },
            })
        }
        "PSETEX" => {
            if items.len() != 4 {
                return Err(anyhow::anyhow!(
                    "wrong number of arguments for 'psetex' command"
                ));
            }

            let key = expect_bulk(&items, 1, "key")?;
            let sec_str = expect_bulk(&items, 2, "time")?;
            let value = expect_bulk(&items, 3, "value")?;

            let sec: u64 = sec_str.parse().unwrap();

            Ok(Command::SET {
                key,
                value: Entry {
                    value: Value::String(value.into_bytes()),
                    expires_at: Some(Instant::now() + Duration::from_millis(sec)),
                },
            })
        }
        "DEL" => {
            let len = items.len();
            if len < 2 {
                return Err(anyhow::anyhow!(
                    "wrong number of arguments for 'del' command"
                ));
            }
            let mut keys: Vec<String> = Vec::new();

            for i in 1..len {
                let key = expect_bulk(&items, i, "key")?;
                keys.push(key);
            }

            Ok(Command::DEL { keys })
        }
        "EXISTS" => {
            let len = items.len();
            if len < 2 {
                return Err(anyhow::anyhow!(
                    "wrong number of arguments for 'exists' command"
                ));
            }
            let mut keys: Vec<String> = Vec::new();

            for i in 1..len {
                let key = expect_bulk(&items, i, "key")?;
                keys.push(key);
            }

            Ok(Command::EXISTS { keys })
        }
        "EXPIRE" => {
            let len = items.len();
            if len < 2 {
                return Err(anyhow::anyhow!(
                    "wrong number of arguments for 'exists' command"
                ));
            }

            let key = expect_bulk(&items, 1, "key")?;
            let sec_str = expect_bulk(&items, 2, "time")?;

            let sec = sec_str.parse().unwrap();

            Ok(Command::EXPIRE { key, sec })
        }
        "TTL" => {
            if items.len() != 2 {
                return Err(anyhow::anyhow!(
                    "wrong number of arguments for 'ttl' command"
                ));
            }

            let key = expect_bulk(&items, 1, "key")?;

            Ok(Command::TTL { key })
        }
        "PTTL" => {
            if items.len() != 2 {
                return Err(anyhow::anyhow!(
                    "wrong number of arguments for 'pttl' command"
                ));
            }

            let key = expect_bulk(&items, 1, "key")?;

            Ok(Command::TTL { key })
        }
        _ => Err(anyhow::anyhow!("unknown command")),
    }
}
