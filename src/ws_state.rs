use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::info;

/// Tracks active viewers per match in memory.
/// No database needed — this is purely real-time state.
#[derive(Clone)]
pub struct ActiveViewers {
    /// match_id → current active viewer count
    counts: Arc<DashMap<String, usize>>,
    /// match_id → broadcast sender for that match's view count
    broadcasters: Arc<DashMap<String, broadcast::Sender<usize>>>,
}

impl ActiveViewers {
    pub fn new() -> Self {
        Self {
            counts: Arc::new(DashMap::new()),
            broadcasters: Arc::new(DashMap::new()),
        }
    }

    /// Subscribe to view count updates for a match.
    /// Returns the current count and a receiver for future updates.
    pub fn subscribe(&self, match_id: &str) -> (usize, broadcast::Receiver<usize>) {
        let entry = self.counts.entry(match_id.to_string()).or_insert(0);
        let current_count = *entry;
        drop(entry);

        let tx = self
            .broadcasters
            .entry(match_id.to_string())
            .or_insert_with(|| {
                let (tx, _rx) = broadcast::channel::<usize>(32);
                tx
            })
            .clone();

        (current_count, tx.subscribe())
    }

    /// Increment the active viewer count for a match.
    /// Returns the new count.
    pub fn increment(&self, match_id: &str) -> usize {
        let mut entry = self.counts.entry(match_id.to_string()).or_insert(0);
        *entry += 1;
        let new_count = *entry;
        drop(entry);

        self.broadcast_count(match_id, new_count);
        info!(match_id, count = new_count, "Viewer connected");
        new_count
    }

    /// Decrement the active viewer count for a match.
    /// Returns the new count (minimum 0).
    pub fn decrement(&self, match_id: &str) -> usize {
        if let Some(mut entry) = self.counts.get_mut(match_id) {
            if *entry > 0 {
                *entry -= 1;
            }
            let new_count = *entry;
            drop(entry);

            self.broadcast_count(match_id, new_count);
            info!(match_id, count = new_count, "Viewer disconnected");
            return new_count;
        }
        0
    }

    /// Get the current active viewer count for a match.
    pub fn get_count(&self, match_id: &str) -> usize {
        self.counts
            .get(match_id)
            .map(|e| *e)
            .unwrap_or(0)
    }

    fn broadcast_count(&self, match_id: &str, count: usize) {
        if let Some(tx) = self.broadcasters.get(match_id) {
            // Ignore errors — no subscribers means nobody cares right now
            let _ = tx.send(count);
        }
    }
}
