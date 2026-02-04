use crate::model::{DB, Entry, Value};
use anyhow::Result;
use std::collections::VecDeque;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

pub async fn lpush_cmd(
    key: String,
    values: Vec<Vec<u8>>,
    _db: &DB,
    socket: &mut TcpStream,
) -> Result<()> {
    let mut db = _db.write().await;

    let len = match db.get_mut(&key) {
        Some(entry) => {
            let list = match entry.value.as_list_mut() {
                Some(l) => l,
                None => {
                    socket
                        .write_all(b"-WRONGTYPE key holds wrong kind of value\r\n")
                        .await?;
                    return Ok(());
                }
            };

            for v in values {
                list.push_front(v);
            }
            list.len()
        }
        None => {
            let mut list = VecDeque::new();
            for v in values {
                list.push_front(v);
            }

            let len = list.len();

            db.insert(
                key,
                Entry {
                    value: Value::List(list),
                    expires_at: None,
                },
            );
            len
        }
    };

    socket.write_all(format!(":{}\r\n", len).as_bytes()).await?;

    Ok(())
}
