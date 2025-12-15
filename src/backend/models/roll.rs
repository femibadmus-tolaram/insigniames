use chrono::Local;
use rusqlite::{params, Connection, Result};
use serde::{Serialize, Deserialize};
use crate::{backend::models::{FilterResponse, Job}, sap::{RollData, post_rolls}};

#[derive(Debug, Serialize)]
pub struct Roll {
    pub id: i32,
    pub output_roll_no: String,
    pub final_meter: f64,
    pub number_of_flags: i32,
    pub flag_reason_id: Option<i32>,
    pub final_weight: f64,
    pub job_id: i32,
    pub created_by: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Deserialize)]
pub struct RollCreatePayload {
    pub output_roll_no: String,
    pub final_meter: f64,
    pub number_of_flags: i32,
    pub flag_reason_id: Option<i32>,
    pub final_weight: f64,
    pub job_id: i32,
}

#[derive(Deserialize)]
pub struct RollPayload {
    pub id: i32,
    pub output_roll_no: Option<String>,
    pub final_meter: Option<f64>,
    pub number_of_flags: Option<i32>,
    pub flag_reason_id: Option<i32>,
    pub final_weight: Option<f64>,
    pub job_id: Option<i32>,
}

#[derive(Deserialize)]
pub struct RollFilterPayload {
    pub job_id: Option<String>,
    pub shift_id: Option<String>,
    pub output_roll_no: Option<String>,
    pub flag_reason_id: Option<String>,
    pub created_by: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub per_page: Option<String>,
    pub page: Option<String>,
    pub status: Option<String>,
}

impl Roll {
    pub fn create(conn: &Connection, data: &RollCreatePayload, user_id: i32) -> Result<Self> {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        conn.execute(
            "INSERT INTO rolls (output_roll_no, final_meter, number_of_flags, flag_reason_id, final_weight, job_id, created_by, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![data.output_roll_no, data.final_meter, data.number_of_flags, data.flag_reason_id, data.final_weight, data.job_id, user_id, now, now],
        )?;
        let id = conn.last_insert_rowid() as i32;
        Ok(Roll {
            id,
            output_roll_no: data.output_roll_no.clone(),
            final_meter: data.final_meter,
            number_of_flags: data.number_of_flags,
            flag_reason_id: data.flag_reason_id,
            final_weight: data.final_weight,
            job_id: data.job_id,
            created_by: user_id,
            created_at: now.clone(),
            updated_at: now.clone(),
        })
    }

    pub async fn update(&mut self, conn: &Connection, data: &RollPayload) -> Result<()> {
        if let Some(final_weight) = data.final_weight {
            let job = Job::find_by_id(conn, self.job_id)?;
            
            let roll_data = RollData {
                alternate_quantity: final_weight.to_string(),
                quantity: self.final_meter.to_string(),
                batch: job.batch_roll_no.clone(),
                production_order: job.production_order.clone(),
            };
            
            let success = post_rolls(roll_data).await;
            if !success {
                return Err(rusqlite::Error::InvalidParameterName("Failed to post roll data".to_string()));
            }
        }
        
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        
        if let Some(output_roll_no) = &data.output_roll_no {
            conn.execute("UPDATE rolls SET output_roll_no = ?1 WHERE id = ?2", params![output_roll_no, self.id])?;
            self.output_roll_no = output_roll_no.clone();
        }
        if let Some(final_meter) = data.final_meter {
            conn.execute("UPDATE rolls SET final_meter = ?1 WHERE id = ?2", params![final_meter, self.id])?;
            self.final_meter = final_meter;
        }
        if let Some(number_of_flags) = data.number_of_flags {
            conn.execute("UPDATE rolls SET number_of_flags = ?1 WHERE id = ?2", params![number_of_flags, self.id])?;
            self.number_of_flags = number_of_flags;
        }
        if let Some(flag_reason_id) = data.flag_reason_id {
            conn.execute("UPDATE rolls SET flag_reason_id = ?1 WHERE id = ?2", params![flag_reason_id, self.id])?;
            self.flag_reason_id = Some(flag_reason_id);
        }
        if let Some(final_weight) = data.final_weight {
            conn.execute("UPDATE rolls SET final_weight = ?1 WHERE id = ?2", params![final_weight, self.id])?;
            self.final_weight = final_weight;
        }
        if let Some(job_id) = data.job_id {
            conn.execute("UPDATE rolls SET job_id = ?1 WHERE id = ?2", params![job_id, self.id])?;
            self.job_id = job_id;
        }
        conn.execute("UPDATE rolls SET updated_at = ?1 WHERE id = ?2", params![now, self.id])?;
        self.updated_at = now;
        Ok(())
    }

    pub fn delete(&self, conn: &Connection) -> Result<()> {
        conn.execute("DELETE FROM rolls WHERE id = ?1", params![self.id])?;
        Ok(())
    }

    pub fn find_by_id(conn: &Connection, id: i32) -> Result<Self> {
        let mut stmt = conn.prepare("SELECT * FROM rolls WHERE id = ?1")?;
        stmt.query_row(params![id], |row| Ok(Roll {
            id: row.get(0)?,
            output_roll_no: row.get(1)?,
            final_meter: row.get(2)?,
            number_of_flags: row.get(3)?,
            flag_reason_id: row.get(4)?,
            final_weight: row.get(5)?,
            job_id: row.get(6)?,
            created_by: row.get(7)?,
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
        }))
    }

    pub fn all(conn: &Connection) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare("SELECT * FROM rolls ORDER BY created_at DESC")?;
        let rolls = stmt.query_map([], |row| Ok(Roll {
            id: row.get(0)?,
            output_roll_no: row.get(1)?,
            final_meter: row.get(2)?,
            number_of_flags: row.get(3)?,
            flag_reason_id: row.get(4)?,
            final_weight: row.get(5)?,
            job_id: row.get(6)?,
            created_by: row.get(7)?,
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
        }))?.collect::<Result<Vec<_>, _>>()?;
        Ok(rolls)
    }

    pub fn filter(conn: &Connection, filter: &RollFilterPayload) -> Result<FilterResponse<Roll>> {
        let mut count_query = "SELECT COUNT(*) FROM rolls r JOIN jobs j ON r.job_id = j.id WHERE 1=1".to_string();
        let mut data_query = "SELECT r.* FROM rolls r JOIN jobs j ON r.job_id = j.id WHERE 1=1".to_string();
        let mut params_vec: Vec<&dyn rusqlite::ToSql> = vec![];

        let mut job_ids: Vec<i32> = vec![];
        let mut flag_reason_ids: Vec<i32> = vec![];
        let mut created_bys: Vec<i32> = vec![];
        let mut output_roll_nos: Vec<String> = vec![];
        let mut start_dates: Vec<String> = vec![];
        let mut end_dates: Vec<String> = vec![];
        let mut shift_ids: Vec<i32> = vec![];
        let mut pages: Vec<i32> = vec![];
        let mut per_pages: Vec<i32> = vec![];

        if let Some(val) = &filter.job_id {
            if let Ok(parsed) = val.parse::<i32>() {
                job_ids.push(parsed);
                params_vec.push(job_ids.last().unwrap());
                count_query.push_str(" AND r.job_id = ?");
                data_query.push_str(" AND r.job_id = ?");
            }
        }

        if let Some(val) = &filter.status {
            if val == "pending" {
                count_query.push_str(" AND r.final_weight = 0");
                data_query.push_str(" AND r.final_weight = 0");
            } else if val == "flagged" {
                count_query.push_str(" AND r.number_of_flags > 0");
                data_query.push_str(" AND r.number_of_flags > 0");
            } else if val == "completed" {
                count_query.push_str(" AND r.final_weight > 0");
                data_query.push_str(" AND r.final_weight > 0");
            }
        }

        if let Some(val) = &filter.output_roll_no {
            if !val.is_empty() {
                output_roll_nos.push(format!("%{}%", val));
                params_vec.push(output_roll_nos.last().unwrap());
                count_query.push_str(" AND r.output_roll_no LIKE ?");
                data_query.push_str(" AND r.output_roll_no LIKE ?");
            }
        }

        if let Some(val) = &filter.flag_reason_id {
            if let Ok(parsed) = val.parse::<i32>() {
                flag_reason_ids.push(parsed);
                params_vec.push(flag_reason_ids.last().unwrap());
                count_query.push_str(" AND r.flag_reason_id = ?");
                data_query.push_str(" AND r.flag_reason_id = ?");
            }
        }

        if let Some(val) = &filter.created_by {
            if let Ok(parsed) = val.parse::<i32>() {
                created_bys.push(parsed);
                params_vec.push(created_bys.last().unwrap());
                count_query.push_str(" AND r.created_by = ?");
                data_query.push_str(" AND r.created_by = ?");
            }
        }

        if let Some(val) = &filter.shift_id {
            if let Ok(parsed) = val.parse::<i32>() {
                shift_ids.push(parsed);
                params_vec.push(shift_ids.last().unwrap());
                count_query.push_str(" AND j.shift_id = ?");
                data_query.push_str(" AND j.shift_id = ?");
            }
        }

        if let Some(val) = &filter.start_date {
            if !val.is_empty() {
                start_dates.push(val.clone());
                params_vec.push(start_dates.last().unwrap());
                count_query.push_str(" AND date(r.created_at) >= date(?)");
                data_query.push_str(" AND date(r.created_at) >= date(?)");
            }
        }

        if let Some(val) = &filter.end_date {
            if !val.is_empty() {
                end_dates.push(val.clone());
                params_vec.push(end_dates.last().unwrap());
                count_query.push_str(" AND date(r.created_at) <= date(?)");
                data_query.push_str(" AND date(r.created_at) <= date(?)");
            }
        }

        let total_count: i32 = conn.query_row(&count_query, params_vec.as_slice(), |row| row.get(0))?;

        data_query.push_str(" ORDER BY r.created_at DESC");

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
            Ok(Roll {
                id: row.get(0)?,
                output_roll_no: row.get(1)?,
                final_meter: row.get(2)?,
                number_of_flags: row.get(3)?,
                flag_reason_id: row.get(4)?,
                final_weight: row.get(5)?,
                job_id: row.get(6)?,
                created_by: row.get(7)?,
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