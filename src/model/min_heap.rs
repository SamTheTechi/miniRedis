use std::{collections::BinaryHeap, sync::Arc, time::Instant};
use tokio::sync::Mutex;

pub type Heap = Arc<Mutex<BinaryHeap<MinHeap>>>;

#[derive(Clone, PartialEq, Eq)]
pub struct MinHeap {
    pub expires_at: Instant,
    pub key: String,
}

impl Ord for MinHeap {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.expires_at.cmp(&self.expires_at)
    }
}

impl PartialOrd for MinHeap {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
