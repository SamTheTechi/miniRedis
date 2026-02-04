use super::bulk_to_string;
use crate::model::RESP;
use anyhow::Result;

pub fn expect_bulk(items: &[RESP], idx: usize, name: &str) -> Result<String> {
    match items.get(idx) {
        Some(RESP::BulkStrings(Some(b))) => {
            bulk_to_string(b).ok_or_else(|| anyhow::anyhow!("invalid {}", name))
        }
        _ => Err(anyhow::anyhow!("invalid {}", name)),
    }
}
