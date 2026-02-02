#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum RESP {
    SimpleStrings(String),
    SimpleErrors(String),
    Integers(i64),
    BulkStrings(Option<Vec<u8>>),
    Arrays(Vec<RESP>),
}
