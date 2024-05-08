use chrono::{DateTime, NaiveDate, Utc};
use rocket::tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{atomic::AtomicU64, Arc},
};

use crate::types::UserID;

#[derive(Default, Serialize, Deserialize)]
pub struct TimeState {
    #[serde(with = "arc_rw_serde")]
    data: Arc<RwLock<HashMap<UserID, TimeSheet>>>,
    id_counter: AtomicU64,
}

mod arc_rw_serde {
    use rocket::{
        serde::{Deserialize, Deserializer, Serialize, Serializer},
        tokio::sync::RwLock,
    };
    use std::sync::Arc;

    pub fn serialize<T, S>(data: &Arc<RwLock<T>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Serialize,
        S: Serializer,
    {
        let (h, _) = crate::get_runtime_handle();
        let data = h.block_on(data.read());
        T::serialize(&data, serializer)
    }

    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<Arc<RwLock<T>>, D::Error>
    where
        T: Deserialize<'de>,
        D: Deserializer<'de>,
    {
        Ok(Arc::new(RwLock::new(T::deserialize(deserializer)?)))
    }
}

impl TimeState {
    pub fn new() -> Self {
        Default::default()
    }

    pub async fn start(&self, user: UserID, note: Option<String>) {
        let mut data = self.data.write().await;
        let sheet = data.entry(user).or_insert_with(|| TimeSheet {
            completed: Vec::new(),
            current: None,
        });
        if sheet.current.is_none() {
            sheet.current = Some(Timestamp {
                time: Utc::now(),
                note,
            });
        }
    }

    pub async fn stop(&self, user: UserID, note: Option<String>) -> Option<u64> {
        let mut data = self.data.write().await;
        let sheet = data.get_mut(&user)?;
        let Some(current) = sheet.current.take() else {
            return None;
        };
        let id = self
            .id_counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        sheet.completed.push(TimeRange {
            start: current,
            end: Timestamp {
                time: Utc::now(),
                note,
            },
            id,
        });
        Some(id)
    }
}

#[derive(Serialize, Deserialize)]
struct TimeSheet {
    completed: Vec<TimeRange>,
    current: Option<Timestamp>,
}

impl TimeSheet {
    fn total_hours(&self) -> f64 {
        let mut total = self.completed.iter().map(TimeRange::hours).sum();
        if let Some(current) = &self.current {
            total += (Utc::now() - current.time).num_seconds() as f64 / 3600.0;
        }
        total
    }

    fn hours_for_day(&self, day: NaiveDate) -> f64 {
        self.completed
            .iter()
            .filter(|range| range.start.time.date_naive() == day)
            .map(TimeRange::hours)
            .sum()
    }

    fn hours_for_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> f64 {
        self.completed
            .iter()
            .filter(|range| range.start.time >= start && range.end.time <= end)
            .map(TimeRange::hours)
            .sum()
    }
}

#[derive(Serialize, Deserialize)]
struct TimeRange {
    id: u64,
    start: Timestamp,
    end: Timestamp,
}

impl From<DateTime<Utc>> for Timestamp {
    fn from(time: DateTime<Utc>) -> Self {
        Timestamp { time, note: None }
    }
}

#[derive(Serialize, Deserialize)]
struct Timestamp {
    time: DateTime<Utc>,
    note: Option<String>,
}

impl TimeRange {
    fn hours(&self) -> f64 {
        (self.end.time - self.start.time).num_seconds() as f64 / 3600.0
    }
}
