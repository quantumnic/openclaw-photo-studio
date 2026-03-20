//! Performance tracking system for monitoring operation timings

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Instant, SystemTime};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerfEvent {
    pub label: String,
    pub duration_ms: u64,
    pub timestamp: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerfStats {
    pub label: String,
    pub count: u64,
    pub total_ms: u64,
    pub avg_ms: f64,
    pub min_ms: u64,
    pub max_ms: u64,
}

#[derive(Debug, Clone)]
pub struct PerfTracker {
    events: Arc<Mutex<Vec<PerfEvent>>>,
    stats: Arc<Mutex<HashMap<String, (u64, u64, u64, u64)>>>, // (count, total, min, max)
}

impl PerfTracker {
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
            stats: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Record a timed operation
    pub fn record<F, R>(label: &str, f: F) -> (R, u64)
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = f();
        let elapsed = start.elapsed().as_millis() as u64;
        (result, elapsed)
    }

    /// Record an event with timing
    pub fn record_event(&self, label: &str, duration_ms: u64) {
        let event = PerfEvent {
            label: label.to_string(),
            duration_ms,
            timestamp: SystemTime::now(),
        };

        // Store event
        if let Ok(mut events) = self.events.lock() {
            events.push(event);

            // Keep only last 1000 events
            if events.len() > 1000 {
                let to_remove = events.len() - 1000;
                events.drain(0..to_remove);
            }
        }

        // Update stats
        if let Ok(mut stats) = self.stats.lock() {
            let entry = stats.entry(label.to_string()).or_insert((0, 0, u64::MAX, 0));
            entry.0 += 1; // count
            entry.1 += duration_ms; // total
            entry.2 = entry.2.min(duration_ms); // min
            entry.3 = entry.3.max(duration_ms); // max
        }
    }

    /// Get performance statistics
    pub fn get_stats(&self) -> Vec<PerfStats> {
        let stats = self.stats.lock().unwrap();
        let mut result: Vec<PerfStats> = stats
            .iter()
            .map(|(label, (count, total, min, max))| PerfStats {
                label: label.clone(),
                count: *count,
                total_ms: *total,
                avg_ms: if *count > 0 {
                    *total as f64 / *count as f64
                } else {
                    0.0
                },
                min_ms: if *min == u64::MAX { 0 } else { *min },
                max_ms: *max,
            })
            .collect();

        // Sort by total time descending
        result.sort_by(|a, b| b.total_ms.cmp(&a.total_ms));
        result
    }

    /// Get recent events
    pub fn get_recent_events(&self, limit: usize) -> Vec<PerfEvent> {
        let events = self.events.lock().unwrap();
        events.iter().rev().take(limit).cloned().collect()
    }

    /// Clear all tracked data
    pub fn clear(&self) {
        if let Ok(mut events) = self.events.lock() {
            events.clear();
        }
        if let Ok(mut stats) = self.stats.lock() {
            stats.clear();
        }
    }
}

impl Default for PerfTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_record_timing() {
        let (result, elapsed) = PerfTracker::record("test_op", || {
            thread::sleep(Duration::from_millis(10));
            42
        });

        assert_eq!(result, 42);
        assert!(elapsed >= 10);
    }

    #[test]
    fn test_record_event() {
        let tracker = PerfTracker::new();

        tracker.record_event("test_op", 100);
        tracker.record_event("test_op", 200);
        tracker.record_event("other_op", 50);

        let stats = tracker.get_stats();
        assert_eq!(stats.len(), 2);

        let test_stat = stats.iter().find(|s| s.label == "test_op").unwrap();
        assert_eq!(test_stat.count, 2);
        assert_eq!(test_stat.total_ms, 300);
        assert_eq!(test_stat.avg_ms, 150.0);
        assert_eq!(test_stat.min_ms, 100);
        assert_eq!(test_stat.max_ms, 200);
    }

    #[test]
    fn test_get_recent_events() {
        let tracker = PerfTracker::new();

        for i in 0..10 {
            tracker.record_event("op", i * 10);
        }

        let recent = tracker.get_recent_events(5);
        assert_eq!(recent.len(), 5);

        // Most recent should be last (90ms)
        assert_eq!(recent[0].duration_ms, 90);
    }

    #[test]
    fn test_clear() {
        let tracker = PerfTracker::new();

        tracker.record_event("test", 100);
        tracker.clear();

        let stats = tracker.get_stats();
        assert_eq!(stats.len(), 0);

        let events = tracker.get_recent_events(10);
        assert_eq!(events.len(), 0);
    }

    #[test]
    fn test_event_limit() {
        let tracker = PerfTracker::new();

        // Record more than 1000 events
        for i in 0..1500 {
            tracker.record_event("test", i);
        }

        let events = tracker.events.lock().unwrap();
        assert_eq!(events.len(), 1000);
    }

    #[test]
    fn test_stats_sorting() {
        let tracker = PerfTracker::new();

        tracker.record_event("fast", 10);
        tracker.record_event("slow", 1000);
        tracker.record_event("medium", 100);

        let stats = tracker.get_stats();

        // Should be sorted by total time descending
        assert_eq!(stats[0].label, "slow");
        assert_eq!(stats[1].label, "medium");
        assert_eq!(stats[2].label, "fast");
    }
}
