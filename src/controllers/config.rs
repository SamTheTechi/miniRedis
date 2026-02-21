use crate::lru::{EvictionPolicy, LruManager};
use anyhow::Result;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

pub async fn config_get_cmd(
    pattern: String,
    lru: &LruManager,
    socket: &mut TcpStream,
) -> Result<()> {
    let pattern = pattern.to_lowercase();
    let mut pairs: Vec<(String, String)> = Vec::new();

    if pattern == "*" || pattern == "maxmemory" {
        pairs.push(("maxmemory".to_string(), lru.maxmemory().to_string()));
    }
    if pattern == "*" || pattern == "maxmemory-policy" {
        let policy = match lru.policy() {
            EvictionPolicy::NoEviction => "noeviction",
            EvictionPolicy::AllKeysLru => "allkeys-lru",
            EvictionPolicy::VolatileTtl => "volatile-ttl",
        };
        pairs.push(("maxmemory-policy".to_string(), policy.to_string()));
    }

    if pairs.is_empty() {
        socket.write_all(b"*0\r\n").await?;
        return Ok(());
    }

    let mut resp = Vec::new();
    resp.extend_from_slice(format!("*{}\r\n", pairs.len() * 2).as_bytes());
    for (k, v) in pairs {
        resp.extend_from_slice(format!("${}\r\n{}\r\n", k.len(), k).as_bytes());
        resp.extend_from_slice(format!("${}\r\n{}\r\n", v.len(), v).as_bytes());
    }

    socket.write_all(&resp).await?;
    Ok(())
}

pub async fn config_set_cmd(
    key: String,
    value: String,
    lru: &LruManager,
    socket: &mut TcpStream,
) -> Result<()> {
    let key = key.to_lowercase();
    match key.as_str() {
        "maxmemory" => {
            let v: usize = value.parse().map_err(|_| anyhow::anyhow!("invalid maxmemory"))?;
            lru.set_maxmemory(v);
            socket.write_all(b"+OK\r\n").await?;
        }
        "maxmemory-policy" => {
            let policy = match value.to_lowercase().as_str() {
                "noeviction" => EvictionPolicy::NoEviction,
                "allkeys-lru" => EvictionPolicy::AllKeysLru,
                "volatile-ttl" => EvictionPolicy::VolatileTtl,
                _ => return Err(anyhow::anyhow!("invalid maxmemory-policy")),
            };
            lru.set_policy(policy);
            socket.write_all(b"+OK\r\n").await?;
        }
        _ => {
            socket
                .write_all(b"-ERR Unsupported CONFIG parameter\r\n")
                .await?;
        }
    }
    Ok(())
}
