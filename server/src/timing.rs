use chrono::{DateTime, NaiveDate, Utc};
use rocket::{
    data::{Data, FromData, Outcome},
    form::FromFormField,
    http::RawStr,
    serde::json::Json,
    tokio::{runtime::Handle, sync::RwLock},
    Request, State,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    ops::Deref,
    sync::{
        atomic::{AtomicU64, AtomicUsize},
        Arc,
    },
};

use chrono::NaiveDateTime;
use chrono::NaiveTime;

// https://stackoverflow.com/questions/25413201/how-do-i-implement-a-trait-i-dont-own-for-a-type-i-dont-own
// https://github.com/SergioBenitez/Rocket/issues/602#issuecomment-380497269
pub struct NaiveDateForm(NaiveDate);
pub struct NaiveTimeForm(NaiveTime);
pub struct NaiveDateTimeForm(NaiveDateTime);

impl Deref for NaiveDateForm {
    type Target = NaiveDate;
    fn deref(&self) -> &NaiveDate {
        &self.0
    }
}

impl Deref for NaiveTimeForm {
    type Target = NaiveTime;
    fn deref(&self) -> &NaiveTime {
        &self.0
    }
}

impl Deref for NaiveDateTimeForm {
    type Target = NaiveDateTime;
    fn deref(&self) -> &NaiveDateTime {
        &self.0
    }
}

impl<'v> FromFormField<'v> for NaiveDateForm {
    fn from_value(field: rocket::form::ValueField<'v>) -> rocket::form::Result<'v, Self> {
        let date = NaiveDate::parse_from_str(field.value, "%Y-%m-%d")
            .map_err(|_| rocket::form::Error::validation("Invalid Date, format: %Y-%m-%d"))?;
        Ok(NaiveDateForm(date))
    }
}

use crate::{auth::Jwt, types::UserID};

#[get("/time?<start>&<end>")]
pub async fn get_time(
    user: Jwt,
    start: Option<NaiveDateForm>,
    end: Option<NaiveDateForm>,
    time_state: &State<TimeState>,
) -> Option<Json<Vec<TimeRange>>> {
    let ts = time_state.data.read().await;
    let user_timesheet = ts.get(&user.name)?;

    if start.is_none() && end.is_none() {
        let times = user_timesheet.completed.clone();
        return Some(Json(times));
    }

    let start = start
        .map(|date| date.0)
        .unwrap_or_else(|| Utc::now().date_naive());
    let end = end
        .map(|date| date.0)
        .unwrap_or_else(|| Utc::now().date_naive());

    let times = user_timesheet.find_in_range(start, end).cloned().collect();

    Some(Json(times))
}

#[derive(Default, Serialize, Deserialize)]
pub struct TimeState {
    #[serde(with = "arc_rw_serde")]
    pub data: Arc<RwLock<HashMap<UserID, TimeSheet>>>,
    id_counter: AtomicUsize,
}

mod arc_rw_serde {
    use crate::run_or_block;
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
        let data = run_or_block(data.read());
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

    pub async fn stop(&self, user: UserID, note: Option<String>) -> Option<usize> {
        let mut data = self.data.write().await;
        let sheet = data.get_mut(&user)?;
        let Some(start) = sheet.current.take() else {
            return None;
        };
        let id = self
            .id_counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        sheet.completed.push(TimeRange {
            start,
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
pub struct TimeSheet {
    completed: Vec<TimeRange>,
    current: Option<Timestamp>,
}

impl TimeSheet {
    fn get(&self, id: usize) -> Option<&TimeRange> {
        self.completed.iter().find(|range| range.id == id)
    }

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

    pub fn find_in_range(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> impl Iterator<Item = &TimeRange> {
        self.completed.iter().filter(move |range| {
            let date = range.start.time.date_naive();
            date >= start && date <= end
        })
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TimeRange {
    id: usize,
    start: Timestamp,
    end: Timestamp,
}

impl From<DateTime<Utc>> for Timestamp {
    fn from(time: DateTime<Utc>) -> Self {
        Timestamp { time, note: None }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Timestamp {
    time: DateTime<Utc>,
    note: Option<String>,
}

impl TimeRange {
    fn hours(&self) -> f64 {
        (self.end.time - self.start.time).num_seconds() as f64 / 3600.0
    }
}
