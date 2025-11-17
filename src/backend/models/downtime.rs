use chrono::Local;
use rusqlite::{params, Connection, Result};
use serde::{Serialize, Deserialize};
use crate::backend::models::FilterResponse;

#[derive(Debug, Serialize)]
pub struct Downtime {
    pub id: i32,
    pub shift_id: i32,
    pub start_time: String,
    pub end_time: String,
    pub duration_minutes: i32,
    pub downtime_reason_id: i32,
    pub created_by: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Deserialize)]
pub struct DowntimeCreatePayload {
    pub shift_id: i32,
    pub start_time: String,
    pub end_time: String,
    pub duration_minutes: i32,
    pub downtime_reason_id: i32,
}

#[derive(Deserialize)]
pub struct DowntimePayload {
    pub id: i32,
    pub shift_id: Option<i32>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub duration_minutes: Option<i32>,
    pub downtime_reason_id: Option<i32>,
}

#[derive(Deserialize)]
pub struct DowntimeFilterPayload {
    pub shift_id: Option<String>,
    pub downtime_reason_id: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub created_by: Option<String>,
    pub per_page: Option<String>,
    pub page: Option<String>,
}

impl Downtime {
    pub fn create(conn: &Connection, data: &DowntimeCreatePayload, user_id: i32) -> Result<Self> {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        conn.execute(
            "INSERT INTO downtimes (shift_id, start_time, end_time, duration_minutes, downtime_reason_id, created_by, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![data.shift_id, data.start_time, data.end_time, data.duration_minutes, data.downtime_reason_id, user_id, now, now],
        )?;
        let id = conn.last_insert_rowid() as i32;
        Ok(Downtime {
            id,
            shift_id: data.shift_id,
            start_time: data.start_time.clone(),
            end_time: data.end_time.clone(),
            duration_minutes: data.duration_minutes,
            downtime_reason_id: data.downtime_reason_id,
            created_by: user_id,
            created_at: now.clone(),
            updated_at: now.clone(),
        })
    }

    pub fn update(&mut self, conn: &Connection, data: &DowntimePayload) -> Result<()> {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        if let Some(shift_id) = data.shift_id {
            conn.execute("UPDATE downtimes SET shift_id = ?1 WHERE id = ?2", params![shift_id, self.id])?;
            self.shift_id = shift_id;
        }
        if let Some(start_time) = &data.start_time {
            conn.execute("UPDATE downtimes SET start_time = ?1 WHERE id = ?2", params![start_time, self.id])?;
            self.start_time = start_time.clone();
        }
        if let Some(end_time) = &data.end_time {
            conn.execute("UPDATE downtimes SET end_time = ?1 WHERE id = ?2", params![end_time, self.id])?;
            self.end_time = end_time.clone();
        }
        if let Some(duration_minutes) = data.duration_minutes {
            conn.execute("UPDATE downtimes SET duration_minutes = ?1 WHERE id = ?2", params![duration_minutes, self.id])?;
            self.duration_minutes = duration_minutes;
        }
        if let Some(downtime_reason_id) = data.downtime_reason_id {
            conn.execute("UPDATE downtimes SET downtime_reason_id = ?1 WHERE id = ?2", params![downtime_reason_id, self.id])?;
            self.downtime_reason_id = downtime_reason_id;
        }
        conn.execute("UPDATE downtimes SET updated_at = ?1 WHERE id = ?2", params![now, self.id])?;
        self.updated_at = now;
        Ok(())
    }

    pub fn delete(&self, conn: &Connection) -> Result<()> {
        conn.execute("DELETE FROM downtimes WHERE id = ?1", params![self.id])?;
        Ok(())
    }

    pub fn find_by_id(conn: &Connection, id: i32) -> Result<Self> {
        let mut stmt = conn.prepare("SELECT * FROM downtimes WHERE id = ?1")?;
        stmt.query_row(params![id], |row| Ok(Downtime {
            id: row.get(0)?,
            shift_id: row.get(1)?,
            start_time: row.get(2)?,
            end_time: row.get(3)?,
            duration_minutes: row.get(4)?,
            downtime_reason_id: row.get(5)?,
            created_by: row.get(6)?,
            created_at: row.get(7)?,
            updated_at: row.get(8)?,
        }))
    }

    pub fn all(conn: &Connection) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare("SELECT * FROM downtimes ORDER BY created_at DESC")?;
        let downtimes = stmt.query_map([], |row| Ok(Downtime {
            id: row.get(0)?,
            shift_id: row.get(1)?,
            start_time: row.get(2)?,
            end_time: row.get(3)?,
            duration_minutes: row.get(4)?,
            downtime_reason_id: row.get(5)?,
            created_by: row.get(6)?,
            created_at: row.get(7)?,
            updated_at: row.get(8)?,
        }))?.collect::<Result<Vec<_>, _>>()?;
        Ok(downtimes)
    }

    pub fn filter(conn: &Connection, filter: &DowntimeFilterPayload) -> Result<FilterResponse<Self>> {
        let mut count_query = "SELECT COUNT(*) FROM downtimes WHERE 1=1".to_string();
        let mut data_query = "SELECT * FROM downtimes WHERE 1=1".to_string();
        let mut params_vec: Vec<&dyn rusqlite::ToSql> = vec![];

        let mut shift_ids: Vec<i32> = vec![];
        let mut downtime_reason_ids: Vec<i32> = vec![];
        let mut created_bys: Vec<i32> = vec![];
        let mut start_dates: Vec<String> = vec![];
        let mut end_dates: Vec<String> = vec![];
        let mut pages: Vec<i32> = vec![];
        let mut per_pages: Vec<i32> = vec![];

        if let Some(val) = &filter.shift_id {
            if let Ok(parsed) = val.parse::<i32>() {
                shift_ids.push(parsed);
                params_vec.push(shift_ids.last().unwrap());
                count_query.push_str(" AND shift_id = ?");
                data_query.push_str(" AND shift_id = ?");
            }
        }

        if let Some(val) = &filter.downtime_reason_id {
            if let Ok(parsed) = val.parse::<i32>() {
                downtime_reason_ids.push(parsed);
                params_vec.push(downtime_reason_ids.last().unwrap());
                count_query.push_str(" AND downtime_reason_id = ?");
                data_query.push_str(" AND downtime_reason_id = ?");
            }
        }

        if let Some(val) = &filter.created_by {
            if let Ok(parsed) = val.parse::<i32>() {
                created_bys.push(parsed);
                params_vec.push(created_bys.last().unwrap());
                count_query.push_str(" AND created_by = ?");
                data_query.push_str(" AND created_by = ?");
            }
        }

        if let Some(val) = &filter.start_date {
            if !val.is_empty() {
                start_dates.push(val.clone());
                params_vec.push(start_dates.last().unwrap());
                count_query.push_str(" AND date(created_at) >= date(?)");
                data_query.push_str(" AND date(created_at) >= date(?)");
            }
        }

        if let Some(val) = &filter.end_date {
            if !val.is_empty() {
                end_dates.push(val.clone());
                params_vec.push(end_dates.last().unwrap());
                count_query.push_str(" AND date(created_at) <= date(?)");
                data_query.push_str(" AND date(created_at) <= date(?)");
            }
        }

        let total_count: i32 = conn.query_row(&count_query, params_vec.as_slice(), |row| row.get(0))?;

        data_query.push_str(" ORDER BY created_at DESC");

        if let (Some(page), Some(per_page)) = (&filter.page, &filter.per_page) {
            if let (Ok(page_val), Ok(per_page_val)) = (page.parse::<i32>(), per_page.parse::<i32>()) {
                if per_page_val > 0 {
                    let offset = (page_val - 1) * per_page_val;
                    pages.push(offset);
                    per_pages.push(per_page_val);
                    data_query.push_str(" LIMIT ? OFFSET ?");
                    params_vec.push(per_pages.last().unwrap());
                    params_vec.push(pages.last().unwrap());
                }
            }
        }

        let mut stmt = conn.prepare(&data_query)?;
        let rows = stmt.query_map(params_vec.as_slice(), |row| {
            Ok(Downtime {
                id: row.get(0)?,
                shift_id: row.get(1)?,
                start_time: row.get(2)?,
                end_time: row.get(3)?,
                duration_minutes: row.get(4)?,
                downtime_reason_id: row.get(5)?,
                created_by: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })?;

        let data = rows.collect::<Result<Vec<_>, _>>()?;

        Ok(FilterResponse {
            total_count,
            data,
        })
    }
}