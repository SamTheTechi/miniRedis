use crate::model::types::{Command, RESP};
use crate::util::bulk_to_string;
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
        "GET" => {
            if items.len() != 2 {
                return Err(anyhow::anyhow!(
                    "wrong number of arguments for 'get' command"
                ));
            }

            let key = match &items[1] {
                RESP::BulkStrings(Some(b)) => {
                    let Some(bs) = bulk_to_string(b) else {
                        return Err(anyhow::anyhow!("invalid key"));
                    };
                    bs
                }
                _ => {
                    return Err(anyhow::anyhow!("invalid key"));
                }
            };

            Ok(Command::GET { key })
        }
        "SET" => {
            if items.len() != 3 {
                return Err(anyhow::anyhow!(
                    "wrong number of arguments for 'set' command"
                ));
            }

            let key = match &items[1] {
                RESP::BulkStrings(Some(b)) => {
                    let Some(bs) = bulk_to_string(b) else {
                        return Err(anyhow::anyhow!("invalid key"));
                    };
                    bs
                }
                _ => {
                    return Err(anyhow::anyhow!("invalid key"));
                }
            };

            Ok(Command::SET {
                key,
                value: items[2].clone(),
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
                let key = match &items[i] {
                    RESP::BulkStrings(Some(b)) => {
                        let Some(bs) = bulk_to_string(b) else {
                            return Err(anyhow::anyhow!("invalid key"));
                        };
                        bs
                    }
                    _ => {
                        return Err(anyhow::anyhow!("invalid key"));
                    }
                };
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
                let key = match &items[i] {
                    RESP::BulkStrings(Some(b)) => {
                        let Some(bs) = bulk_to_string(b) else {
                            return Err(anyhow::anyhow!("invalid key"));
                        };
                        bs
                    }
                    _ => {
                        return Err(anyhow::anyhow!("invalid key"));
                    }
                };
                keys.push(key);
            }

            Ok(Command::EXISTS { keys })
        }
        _ => Err(anyhow::anyhow!("unknown command")),
    }
}
