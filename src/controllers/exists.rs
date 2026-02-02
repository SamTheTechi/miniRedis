use crate::model::DB;
use crate::util::is_expired;
use anyhow::Result;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

pub async fn exists_cmd(keys: Vec<String>, _db: &DB, socket: &mut TcpStream) -> Result<()> {
    let mut removed_count = 0;
    {
        let db = _db.read().await;

        for key in keys {
            if let Some(entry) = db.get(&key) {
                if !is_expired(entry) {
                    removed_count += 1;
                }
            }
        }
    }

    socket
        .write_all(format!(":{}\r\n", removed_count).as_bytes())
        .await?;
    Ok(())
}
