use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub enum RESP {
    SimpleStrings(String),
    SimpleErrors(String),
    Integers(i64),
    BulkStrings(Option<Vec<u8>>),
    Arrays(Vec<RESP>),
}

pub type DB = Arc<Mutex<HashMap<String, RESP>>>;
