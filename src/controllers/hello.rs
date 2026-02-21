use anyhow::Result;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

pub async fn hello_cmd(version: Option<u8>, socket: &mut TcpStream) -> Result<()> {
    if let Some(v) = version {
        if v != 2 && v != 3 {
            socket
                .write_all(b"-ERR unsupported HELLO version\r\n")
                .await?;
            return Ok(());
        }
    }

    let parts = [
        ("server", "miniRedis"),
        ("version", "0.1.0"),
        ("proto", "2"),
        ("id", "0"),
        ("mode", "standalone"),
    ];

    let mut resp = Vec::new();
    resp.extend_from_slice(format!("*{}\r\n", parts.len() * 2).as_bytes());
    for (k, v) in parts {
        resp.extend_from_slice(format!("${}\r\n{}\r\n", k.len(), k).as_bytes());
        resp.extend_from_slice(format!("${}\r\n{}\r\n", v.len(), v).as_bytes());
    }

    socket.write_all(&resp).await?;
    Ok(())
}
