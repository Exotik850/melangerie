use chrono::{DateTime, NaiveDate, Utc};
use rocket::{form::FromFormField, serde::json::Json, tokio::sync::RwLock};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    ops::Deref,
    sync::{atomic::AtomicI64, Arc},
};

use chrono::NaiveDateTime;
use chrono::NaiveTime;

// type AllowedTimeDB = HashMap<UserID, HashSet<UserID>>;

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

use crate::{auth::Jwt, types::UserID, SqliteDB};

#[get("/time?<start>&<end>")]
pub async fn get_time(
    user: Jwt,
    start: Option<NaiveDateForm>,
    end: Option<NaiveDateForm>,
    db: SqliteDB,
) -> Option<Json<Vec<TimeRange>>> {
    // let ts = server_state.time.data.read().await;
    // let user_timesheet = ts.get(&user.name)?;
    let mut time_range = db
        .run(move |d| {
            d.prepare(
                "
          SELECT te.timesheet_id, te.start_time, te.start_note, te.end_time, te.end_note
          FROM timesheets ts
          INNER JOIN time_entries te ON ts.timesheet_id = te.timesheet_id
          WHERE ts.user_id = ?
      ",
            )?
            .query_map(params![user.name.0], |row| {
                Ok(TimeRange {
                    id: row.get(0)?,
                    start: Timestamp {
                        time: row.get(1)?,
                        note: row.get(2)?,
                    },
                    end: Timestamp {
                        time: row.get(3)?,
                        note: row.get(4)?,
                    },
                })
            })?
            .collect::<Result<Vec<TimeRange>, _>>()
        })
        .await
        .inspect_err(|e| log::error!("Error getting times: {e}"))
        .ok()?;

    if start.is_none() && end.is_none() {
        return Some(Json(time_range));
    }

    let start = start
        .map(|date| date.0)
        .unwrap_or_else(|| Utc::now().date_naive());
    let end = end
        .map(|date| date.0)
        .unwrap_or_else(|| Utc::now().date_naive());

    time_range.retain(|t| start >= t.start.time.date_naive() && t.end.time.date_naive() <= end);

    Some(Json(time_range))
}

#[derive(Default, Clone)]
pub struct TimeState {
    pub data: Arc<RwLock<HashMap<UserID, TimeSheet>>>,
    id_counter: Arc<AtomicI64>,
}

impl TimeState {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn from_data(data: HashMap<UserID, TimeSheet>) -> Self {
        let max_id = data
            .values()
            .flat_map(|sheet| sheet.completed.iter())
            .map(|range| range.id)
            .max()
            .unwrap_or(0);
        TimeState {
            data: Arc::new(RwLock::new(data)),
            id_counter: Arc::new(AtomicI64::new(max_id)),
        }
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

    pub async fn stop(&self, user: &UserID, note: Option<String>) -> Option<i64> {
        let mut data = self.data.write().await;
        let sheet = data.get_mut(user)?;
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

    pub async fn is_active(&self, user: &UserID) -> Option<bool> {
        self.data
            .read()
            .await
            .get(user)
            .map(|sheet| sheet.current.is_some())
    }
}

#[derive(Serialize, Deserialize)]
pub struct TimeSheet {
    completed: Vec<TimeRange>,
    current: Option<Timestamp>,
}

impl TimeSheet {
    fn get(&self, id: i64) -> Option<&TimeRange> {
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
    id: i64,
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
