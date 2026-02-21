use crate::{
    model::{Command, RESP},
    util::{bulk_to_string, expect_bulk},
};
use anyhow::{Ok, Result};

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
        "QUIT" => Ok(Command::QUIT),
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
            let value = expect_bulk(&items, 2, "value")?;

            Ok(Command::SET {
                key,
                value: value.into_bytes(),
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

            Ok(Command::SETEX {
                key,
                value: value.into_bytes(),
                seconds: sec,
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

            Ok(Command::PSETEX {
                key,
                value: value.into_bytes(),
                seconds: sec,
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
            if len != 3 {
                return Err(anyhow::anyhow!(
                    "wrong number of arguments for 'expire' command"
                ));
            }

            let key = expect_bulk(&items, 1, "key")?;
            let sec_str = expect_bulk(&items, 2, "time")?;

            let sec = sec_str.parse().unwrap();

            Ok(Command::EXPIRE { key, seconds: sec })
        }
        "PERSIST" => {
            if items.len() != 2 {
                return Err(anyhow::anyhow!(
                    "wrong number of arguments for 'persist' command"
                ));
            }

            let key = expect_bulk(&items, 1, "key")?;

            Ok(Command::PERSIST { key })
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
        "TYPE" => {
            if items.len() != 2 {
                return Err(anyhow::anyhow!(
                    "wrong number of arguments for 'type' command"
                ));
            }

            let key = expect_bulk(&items, 1, "key")?;

            Ok(Command::TYPE { key })
        }
        "INFO" => {
            if items.len() > 2 {
                return Err(anyhow::anyhow!(
                    "wrong number of arguments for 'info' command"
                ));
            }

            let section = if items.len() == 2 {
                Some(expect_bulk(&items, 1, "section")?)
            } else {
                None
            };

            Ok(Command::INFO { section })
        }
        "HELLO" => {
            if items.len() > 2 {
                return Err(anyhow::anyhow!(
                    "wrong number of arguments for 'hello' command"
                ));
            }
            let version = if items.len() == 2 {
                let v = expect_bulk(&items, 1, "version")?;
                let v = v.parse::<u8>().map_err(|_| anyhow::anyhow!("invalid version"))?;
                Some(v)
            } else {
                None
            };
            Ok(Command::HELLO { version })
        }
        "COMMAND" => Ok(Command::COMMAND),
        "CLIENT" => {
            if items.len() >= 2 {
                let sub = expect_bulk(&items, 1, "subcommand")?.to_uppercase();
                if sub == "SETINFO" {
                    return Ok(Command::ClientSetinfo);
                }
            }
            Err(anyhow::anyhow!("unsupported client subcommand"))
        }
        "CONFIG" => {
            if items.len() < 3 {
                return Err(anyhow::anyhow!(
                    "wrong number of arguments for 'config' command"
                ));
            }

            let sub = expect_bulk(&items, 1, "subcommand")?.to_uppercase();
            match sub.as_str() {
                "GET" => {
                    if items.len() != 3 {
                        return Err(anyhow::anyhow!(
                            "wrong number of arguments for 'config get' command"
                        ));
                    }
                    let pattern = expect_bulk(&items, 2, "pattern")?;
                    Ok(Command::ConfigGet { pattern })
                }
                "SET" => {
                    if items.len() != 4 {
                        return Err(anyhow::anyhow!(
                            "wrong number of arguments for 'config set' command"
                        ));
                    }
                    let key = expect_bulk(&items, 2, "parameter")?;
                    let value = expect_bulk(&items, 3, "value")?;
                    Ok(Command::ConfigSet { key, value })
                }
                _ => Err(anyhow::anyhow!("unsupported config subcommand")),
            }
        }
        "PTTL" => {
            if items.len() != 2 {
                return Err(anyhow::anyhow!(
                    "wrong number of arguments for 'pttl' command"
                ));
            }

            let key = expect_bulk(&items, 1, "key")?;

            Ok(Command::PTTL { key })
        }
        "LPUSH" => {
            let len = items.len();
            if len < 3 {
                return Err(anyhow::anyhow!(
                    "wrong number of arguments for 'lpush' command"
                ));
            }

            let key = expect_bulk(&items, 1, "key")?;
            let mut values: Vec<Vec<u8>> = Vec::new();

            for i in 2..len {
                let value = expect_bulk(&items, i, "value")?;
                values.push(value.into_bytes());
            }

            Ok(Command::LPUSH { key, values })
        }
        "RPUSH" => {
            let len = items.len();
            if len < 3 {
                return Err(anyhow::anyhow!(
                    "wrong number of arguments for 'rpush' command"
                ));
            }

            let key = expect_bulk(&items, 1, "key")?;
            let mut values: Vec<Vec<u8>> = Vec::new();

            for i in 2..len {
                let value = expect_bulk(&items, i, "value")?;
                values.push(value.into_bytes());
            }

            Ok(Command::RPUSH { key, values })
        }
        "LPOP" => {
            let len = items.len();
            if len != 2 {
                return Err(anyhow::anyhow!(
                    "wrong number of arguments for 'lpop' command"
                ));
            }

            let key = expect_bulk(&items, 1, "key")?;

            Ok(Command::LPOP { key })
        }
        "RPOP" => {
            let len = items.len();
            if len != 2 {
                return Err(anyhow::anyhow!(
                    "wrong number of arguments for 'rpop' command"
                ));
            }

            let key = expect_bulk(&items, 1, "key")?;

            Ok(Command::RPOP { key })
        }
        _ => Err(anyhow::anyhow!("unknown command")),
    }
}
