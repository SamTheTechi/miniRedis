mod client;
mod controllers;
mod model;
mod parser;
mod util;

use crate::client::process_client;
use crate::model::types::{DB, RESP};
use anyhow::Result;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;
    let db: DB = Arc::new(Mutex::new(HashMap::<String, RESP>::new()));

    loop {
        let (socket, _) = listener.accept().await.unwrap();

        let db = db.clone();

        tokio::spawn(async move {
            if let Err(e) = process_client(socket, db).await {
                eprintln!("Error: {:?}", e);
            }
        });
    }
}
