use crate::model::{DB, Entry};
use anyhow::Result;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

pub async fn set_cmd(key: String, value: Entry, _db: &DB, socket: &mut TcpStream) -> Result<()> {
    let mut db = _db.write().await;
    db.insert(key, value);
    socket.write_all(b"+OK\r\n").await?;

    Ok(())
}
