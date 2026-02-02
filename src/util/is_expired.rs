use std::time::Instant;

use crate::model::Entry;

pub fn is_expired(entry: &Entry) -> bool {
    match entry.expires_at {
        Some(t) => Instant::now() >= t,
        None => false,
    }
}
