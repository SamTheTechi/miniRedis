use crate::model::CommandInfo;
use anyhow::Result;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

pub async fn command_cmd(socket: &mut TcpStream) -> Result<()> {
    let commands: Vec<CommandInfo> = vec![
        CommandInfo::new("ping", 1, &["fast"], 0, 0, 0),
        CommandInfo::new("quit", 1, &["fast"], 0, 0, 0),
        CommandInfo::new("get", 2, &["readonly", "fast"], 1, 1, 1),
        CommandInfo::new("set", -3, &["write"], 1, 1, 1),
        CommandInfo::new("setex", 4, &["write"], 1, 1, 1),
        CommandInfo::new("psetex", 4, &["write"], 1, 1, 1),
        CommandInfo::new("del", -2, &["write"], 1, -1, 1),
        CommandInfo::new("exists", -2, &["readonly", "fast"], 1, -1, 1),
        CommandInfo::new("expire", 3, &["write", "fast"], 1, 1, 1),
        CommandInfo::new("persist", 2, &["write", "fast"], 1, 1, 1),
        CommandInfo::new("ttl", 2, &["readonly", "fast"], 1, 1, 1),
        CommandInfo::new("pttl", 2, &["readonly", "fast"], 1, 1, 1),
        CommandInfo::new("type", 2, &["readonly", "fast"], 1, 1, 1),
        CommandInfo::new("lpush", -3, &["write"], 1, 1, 1),
        CommandInfo::new("rpush", -3, &["write"], 1, 1, 1),
        CommandInfo::new("lpop", 2, &["write", "fast"], 1, 1, 1),
        CommandInfo::new("rpop", 2, &["write", "fast"], 1, 1, 1),
        CommandInfo::new("config", -2, &["admin", "readonly"], 0, 0, 0),
        CommandInfo::new("info", -1, &["readonly"], 0, 0, 0),
        CommandInfo::new("hello", -1, &["readonly", "fast"], 0, 0, 0),
        CommandInfo::new("command", 0, &["readonly"], 0, 0, 0),
        CommandInfo::new("client", -2, &["readonly"], 0, 0, 0),
    ];

    let mut resp = Vec::new();
    resp.extend_from_slice(&crate::util::array_len(commands.len()));
    for cmd in commands {
        resp.extend_from_slice(&cmd.to_resp());
    }

    socket.write_all(&resp).await?;
    Ok(())
}
