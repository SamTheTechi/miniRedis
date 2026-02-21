mod async_heap_delete;
mod controllers;
mod handle_client;
mod lru;
mod model;
mod parser;
mod util;

use crate::handle_client::process_client;
use crate::model::{DB, Entry, Heap, MinHeap};
use crate::{
    async_heap_delete::async_clean_db_heap,
    lru::{EvictionPolicy, LruManager},
};
use anyhow::Result;
use std::{
    collections::{BinaryHeap, HashMap},
    env,
    sync::Arc,
};
use tokio::{net::TcpListener, sync::Mutex, sync::RwLock};

#[tokio::main]
async fn main() -> Result<()> {
    let mut bind_addr = "127.0.0.1".to_string();
    let mut port: u16 = 6379;
    let mut maxmemory = env::var("MINIREDIS_MAXMEMORY")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(0);
    let mut policy = env::var("MINIREDIS_MAXMEMORY_POLICY")
        .unwrap_or_else(|_| "noeviction".to_string())
        .to_lowercase();

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--bind" => {
                if let Some(v) = args.next() {
                    bind_addr = v;
                }
            }
            "--port" => {
                if let Some(v) = args.next() {
                    if let Ok(p) = v.parse::<u16>() {
                        port = p;
                    }
                }
            }
            "--maxmemory" => {
                if let Some(v) = args.next() {
                    if let Ok(m) = v.parse::<usize>() {
                        maxmemory = m;
                    }
                }
            }
            "--maxmemory-policy" => {
                if let Some(v) = args.next() {
                    policy = v.to_lowercase();
                }
            }
            "--help" | "-h" => {
                println!(
                    "miniRedis options:\n  --bind <ip>\n  --port <port>\n  --maxmemory <bytes>\n  --maxmemory-policy <noeviction|allkeys-lru|volatile-ttl>"
                );
                return Ok(());
            }
            _ => {}
        }
    }

    let policy = match policy.as_str() {
        "allkeys-lru" => EvictionPolicy::AllKeysLru,
        "volatile-ttl" => EvictionPolicy::VolatileTtl,
        "noeviction" => EvictionPolicy::NoEviction,
        _ => EvictionPolicy::NoEviction,
    };

    let bind = format!("{}:{}", bind_addr, port);
    let listener = TcpListener::bind(&bind).await?;
    println!("miniRedis listening on {}", bind);
    let db: DB = Arc::new(RwLock::new(HashMap::<String, Entry>::new()));
    let heap: Heap = Arc::new(Mutex::new(BinaryHeap::<MinHeap>::new()));
    let lru = LruManager::new(maxmemory, policy);

    async_clean_db_heap(db.clone(), heap.clone(), lru.clone());

    loop {
        let (socket, _) = listener.accept().await.unwrap();

        let db = db.clone();
        let heap = heap.clone();

        let lru = lru.clone();

        tokio::spawn(async move {
            if let Err(e) = process_client(socket, db, heap, lru).await {
                eprintln!("Error: {:?}", e);
            }
        });
    }
}
