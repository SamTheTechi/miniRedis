mod async_heap_delete;
mod controllers;
mod handle_client;
mod model;
mod parser;
mod util;

use crate::async_heap_delete::async_clean_db_heap;
use crate::handle_client::process_client;
use crate::model::{DB, Entry, Heap, MinHeap};
use anyhow::Result;
use std::{
    collections::{BinaryHeap, HashMap},
    sync::Arc,
};
use tokio::{net::TcpListener, sync::Mutex, sync::RwLock};

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;
    let db: DB = Arc::new(RwLock::new(HashMap::<String, Entry>::new()));
    let heap: Heap = Arc::new(Mutex::new(BinaryHeap::<MinHeap>::new()));

    async_clean_db_heap(db.clone(), heap.clone());

    loop {
        let (socket, _) = listener.accept().await.unwrap();

        let db = db.clone();
        let heap = heap.clone();

        tokio::spawn(async move {
            if let Err(e) = process_client(socket, db, heap).await {
                eprintln!("Error: {:?}", e);
            }
        });
    }
}
