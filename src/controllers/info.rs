use crate::lru::{EvictionPolicy, LruManager};
use crate::model::DB;
use anyhow::Result;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

pub async fn info_cmd(
    section: Option<String>,
    db: &DB,
    lru: &LruManager,
    socket: &mut TcpStream,
) -> Result<()> {
    let section = section.map(|s| s.to_lowercase());
    let key_count = db.read().await.len();
    let used = lru.used_bytes();
    let maxmemory = lru.maxmemory();
    let policy = match lru.policy() {
        EvictionPolicy::NoEviction => "noeviction",
        EvictionPolicy::AllKeysLru => "allkeys-lru",
        EvictionPolicy::VolatileTtl => "volatile-ttl",
    };

    let mut out = String::new();

    let want_all = section.is_none();
    let want = |name: &str| want_all || section.as_deref() == Some(name);

    if want("server") {
        out.push_str("# Server\r\n");
        out.push_str("redis_version:0.0.0\r\n");
    }
    if want("clients") {
        out.push_str("# Clients\r\n");
        out.push_str("connected_clients:0\r\n");
    }
    if want("memory") {
        out.push_str("# Memory\r\n");
        out.push_str(&format!("used_memory:{}\r\n", used));
        out.push_str(&format!("maxmemory:{}\r\n", maxmemory));
        out.push_str(&format!("maxmemory_policy:{}\r\n", policy));
    }
    if want("stats") {
        out.push_str("# Stats\r\n");
        out.push_str(&format!("keys:{}\r\n", key_count));
    }

    let mut resp = Vec::new();
    resp.extend_from_slice(b"$");
    resp.extend_from_slice(out.len().to_string().as_bytes());
    resp.extend_from_slice(b"\r\n");
    resp.extend_from_slice(out.as_bytes());
    resp.extend_from_slice(b"\r\n");

    socket.write_all(&resp).await?;
    Ok(())
}
