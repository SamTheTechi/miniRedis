use crate::{
    lru::{estimate_entry_bytes, LruManager},
    model::DB,
};
use anyhow::Result;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

pub async fn del_cmd(
    keys: Vec<String>,
    _db: &DB,
    lru: &LruManager,
    socket: &mut TcpStream,
) -> Result<()> {
    let mut removed_count = 0;
    let mut removed_bytes = 0usize;
    let mut removed_keys: Vec<String> = Vec::new();
    {
        let mut db = _db.write().await;

        for key in &keys {
            if let Some((stored_key, entry)) = db.remove_entry(key.as_str()) {
                removed_count += 1;
                removed_bytes += estimate_entry_bytes(&stored_key, &entry);
                removed_keys.push(stored_key);
            }
        }
    }

    socket
        .write_all(format!(":{}\r\n", removed_count).as_bytes())
        .await?;
    if removed_bytes > 0 {
        lru.adjust_used_bytes(-(removed_bytes as isize));
    }
    lru.remove_keys(&removed_keys).await;
    Ok(())
}
