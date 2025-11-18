use chrono::Local;
use rusqlite::{params, Connection, Result};
use serde::{Serialize, Deserialize};
use crate::backend::models::FilterResponse;

#[derive(Debug, Serialize)]
pub struct Job {
    pub id: i32,
    pub shift_id: i32,
    pub production_order: String,
    pub batch_roll_no: String,
    pub start_weight: f64,
    pub start_meter: f64,
    pub created_by: i32,
    pub machine_id: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Deserialize)]
pub struct JobCreatePayload {
    pub shift_id: i32,
    pub production_order: String,
    pub batch_roll_no: String,
    pub start_weight: f64,
    pub start_meter: f64,
    pub machine_id: i32,
}

#[derive(Deserialize)]
pub struct JobPayload {
    pub id: i32,
    pub shift_id: Option<i32>,
    pub production_order: Option<String>,
    pub batch_roll_no: Option<String>,
    pub start_weight: Option<f64>,
    pub start_meter: Option<f64>,
    pub machine_id: Option<i32>,
}

#[derive(Deserialize)]
pub struct JobFilterPayload {
    pub shift_id: Option<String>,
    pub production_order: Option<String>,
    pub batch_roll_no: Option<String>,
    pub created_by: Option<String>,
    pub machine_id: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub per_page: Option<String>,
    pub page: Option<String>,
}

impl Job {
    pub fn create(conn: &Connection, data: &JobCreatePayload, user_id: i32) -> Result<Self> {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        conn.execute(
            "INSERT INTO jobs (shift_id, production_order, batch_roll_no, start_weight, start_meter, created_by, machine_id, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![data.shift_id, data.production_order, data.batch_roll_no, data.start_weight, data.start_meter, user_id, data.machine_id, now, now],
        )?;
        let id = conn.last_insert_rowid() as i32;
        Ok(Job {
            id,
            shift_id: data.shift_id,
            production_order: data.production_order.clone(),
            batch_roll_no: data.batch_roll_no.clone(),
            start_weight: data.start_weight,
            start_meter: data.start_meter,
            created_by: user_id,
            machine_id: data.machine_id,
            created_at: now.clone(),
            updated_at: now.clone(),
        })
    }

    pub fn update(&mut self, conn: &Connection, data: &JobPayload) -> Result<()> {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        if let Some(shift_id) = data.shift_id {
            conn.execute("UPDATE jobs SET shift_id = ?1 WHERE id = ?2", params![shift_id, self.id])?;
            self.shift_id = shift_id;
        }
        if let Some(production_order) = &data.production_order {
            conn.execute("UPDATE jobs SET production_order = ?1 WHERE id = ?2", params![production_order, self.id])?;
            self.production_order = production_order.clone();
        }
        if let Some(batch_roll_no) = &data.batch_roll_no {
            conn.execute("UPDATE jobs SET batch_roll_no = ?1 WHERE id = ?2", params![batch_roll_no, self.id])?;
            self.batch_roll_no = batch_roll_no.clone();
        }
        if let Some(start_weight) = data.start_weight {
            conn.execute("UPDATE jobs SET start_weight = ?1 WHERE id = ?2", params![start_weight, self.id])?;
            self.start_weight = start_weight;
        }
        if let Some(start_meter) = data.start_meter {
            conn.execute("UPDATE jobs SET start_meter = ?1 WHERE id = ?2", params![start_meter, self.id])?;
            self.start_meter = start_meter;
        }
        if let Some(machine_id) = data.machine_id {
            conn.execute("UPDATE jobs SET machine_id = ?1 WHERE id = ?2", params![machine_id, self.id])?;
            self.machine_id = machine_id;
        }
        conn.execute("UPDATE jobs SET updated_at = ?1 WHERE id = ?2", params![now, self.id])?;
        self.updated_at = now;
        Ok(())
    }

    pub fn delete(&self, conn: &Connection) -> Result<()> {
        conn.execute("DELETE FROM jobs WHERE id = ?1", params![self.id])?;
        Ok(())
    }

    pub fn find_by_id(conn: &Connection, id: i32) -> Result<Self> {
        let mut stmt = conn.prepare("SELECT * FROM jobs WHERE id = ?1")?;
        stmt.query_row(params![id], |row| Ok(Job {
            id: row.get(0)?,
            shift_id: row.get(1)?,
            production_order: row.get(2)?,
            batch_roll_no: row.get(3)?,
            start_weight: row.get(4)?,
            start_meter: row.get(5)?,
            created_by: row.get(6)?,
            machine_id: row.get(7)?,
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
        }))
    }

    pub fn all(conn: &Connection) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare("SELECT * FROM jobs ORDER BY created_at DESC")?;
        let jobs = stmt.query_map([], |row| Ok(Job {
            id: row.get(0)?,
            shift_id: row.get(1)?,
            production_order: row.get(2)?,
            batch_roll_no: row.get(3)?,
            start_weight: row.get(4)?,
            start_meter: row.get(5)?,
            created_by: row.get(6)?,
            machine_id: row.get(7)?,
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
        }))?.collect::<Result<Vec<_>, _>>()?;
        Ok(jobs)
    }

    pub fn filter(conn: &Connection, filter: &JobFilterPayload) -> Result<FilterResponse<Self>> {
        let mut count_query = "SELECT COUNT(*) FROM jobs WHERE 1=1".to_string();
        let mut data_query = "SELECT * FROM jobs WHERE 1=1".to_string();
        let mut params_vec: Vec<&dyn rusqlite::ToSql> = vec![];

        let mut shift_ids: Vec<i32> = vec![];
        let mut created_bys: Vec<i32> = vec![];
        let mut machine_ids: Vec<i32> = vec![];
        let mut production_orders: Vec<String> = vec![];
        let mut batch_roll_nos: Vec<String> = vec![];
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

        if let Some(val) = &filter.production_order {
            if !val.is_empty() {
                production_orders.push(format!("%{}%", val));
                params_vec.push(production_orders.last().unwrap());
                count_query.push_str(" AND production_order LIKE ?");
                data_query.push_str(" AND production_order LIKE ?");
            }
        }

        if let Some(val) = &filter.batch_roll_no {
            if !val.is_empty() {
                batch_roll_nos.push(format!("%{}%", val));
                params_vec.push(batch_roll_nos.last().unwrap());
                count_query.push_str(" AND batch_roll_no LIKE ?");
                data_query.push_str(" AND batch_roll_no LIKE ?");
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

        if let Some(val) = &filter.machine_id {
            if let Ok(parsed) = val.parse::<i32>() {
                machine_ids.push(parsed);
                params_vec.push(machine_ids.last().unwrap());
                count_query.push_str(" AND machine_id = ?");
                data_query.push_str(" AND machine_id = ?");
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
            Ok(Job {
                id: row.get(0)?,
                shift_id: row.get(1)?,
                production_order: row.get(2)?,
                batch_roll_no: row.get(3)?,
                start_weight: row.get(4)?,
                start_meter: row.get(5)?,
                created_by: row.get(6)?,
                machine_id: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })?;

        let data = rows.collect::<Result<Vec<_>, _>>()?;

        Ok(FilterResponse {
            total_count,
            data,
        })
    }
}