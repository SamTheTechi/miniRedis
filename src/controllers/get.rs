use crate::model::types::DB;
use anyhow::Result;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

pub async fn get_cmd(key: String, _db: &DB, socket: &mut TcpStream) -> Result<()> {
    let db = _db.read().await;
    match db.get(&key) {
        Some(v) => socket.write_all(&v.to_bytes()).await?,
        None => socket.write_all(b"$-1\r\n").await?,
    }

    Ok(())
}
