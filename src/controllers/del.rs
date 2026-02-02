use crate::model::DB;
use anyhow::Result;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

pub async fn del_cmd(keys: Vec<String>, _db: &DB, socket: &mut TcpStream) -> Result<()> {
    let mut removed_count = 0;
    {
        let mut db = _db.write().await;

        for key in keys {
            if db.remove(&key).is_some() {
                removed_count += 1;
            }
        }
    }

    socket
        .write_all(format!(":{}\r\n", removed_count).as_bytes())
        .await?;
    Ok(())
}
