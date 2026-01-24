use chrono::Local;
use rusqlite::{params, Connection, Result};
use serde::{Serialize, Deserialize};
use crate::backend::models::FilterResponse;

#[derive(Debug, Serialize)]
pub struct ActualInkUsage {
    pub id: i32,
    pub shift_id: i32,
    pub colour_id: i32,
    pub batch_code: String,
    pub kgs_issued: f64,
    pub created_by: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Deserialize)]
pub struct ActualInkUsageCreatePayload {
    pub shift_id: i32,
    pub colour_id: i32,
    pub batch_code: String,
    pub kgs_issued: f64,
}

#[derive(Deserialize)]
pub struct ActualInkUsagePayload {
    pub id: i32,
    pub shift_id: Option<i32>,
    pub colour_id: Option<i32>,
    pub batch_code: Option<String>,
    pub kgs_issued: Option<f64>,
}

#[derive(Deserialize)]
pub struct ActualInkUsageFilterPayload {
    pub shift_id: Option<String>,
    pub colour_id: Option<String>,
    pub batch_code: Option<String>,
    pub created_by: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub per_page: Option<String>,
    pub page: Option<String>,
}

impl ActualInkUsage {
    pub fn create(conn: &Connection, data: &ActualInkUsageCreatePayload, user_id: i32) -> Result<Self> {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        conn.execute(
            "INSERT INTO ink_usages (shift_id, colour_id, batch_code, kgs_issued, created_by, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![data.shift_id, data.colour_id, data.batch_code, data.kgs_issued, user_id, now, now],
        )?;
        let id = conn.last_insert_rowid() as i32;
        Ok(ActualInkUsage {
            id,
            shift_id: data.shift_id,
            colour_id: data.colour_id,
            batch_code: data.batch_code.clone(),
            kgs_issued: data.kgs_issued,
            created_by: user_id,
            created_at: now.clone(),
            updated_at: now.clone(),
        })
    }

    pub fn update(&mut self, conn: &Connection, data: &ActualInkUsagePayload) -> Result<()> {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        if let Some(shift_id) = data.shift_id {
            conn.execute("UPDATE ink_usages SET shift_id = ?1 WHERE id = ?2", params![shift_id, self.id])?;
            self.shift_id = shift_id;
        }
        if let Some(colour_id) = data.colour_id {
            conn.execute("UPDATE ink_usages SET colour_id = ?1 WHERE id = ?2", params![colour_id, self.id])?;
            self.colour_id = colour_id;
        }
        if let Some(batch_code) = &data.batch_code {
            conn.execute("UPDATE ink_usages SET batch_code = ?1 WHERE id = ?2", params![batch_code, self.id])?;
            self.batch_code = batch_code.clone();
        }
        if let Some(kgs_issued) = data.kgs_issued {
            conn.execute("UPDATE ink_usages SET kgs_issued = ?1 WHERE id = ?2", params![kgs_issued, self.id])?;
            self.kgs_issued = kgs_issued;
        }
        conn.execute("UPDATE ink_usages SET updated_at = ?1 WHERE id = ?2", params![now, self.id])?;
        self.updated_at = now;
        Ok(())
    }

    pub fn delete(&self, conn: &Connection) -> Result<()> {
        conn.execute("DELETE FROM ink_usages WHERE id = ?1", params![self.id])?;
        Ok(())
    }

    pub fn find_by_id(conn: &Connection, id: i32) -> Result<Self> {
        let mut stmt = conn.prepare("SELECT * FROM ink_usages WHERE id = ?1")?;
        stmt.query_row(params![id], |row| Ok(ActualInkUsage {
            id: row.get(0)?,
            shift_id: row.get(1)?,
            colour_id: row.get(2)?,
            batch_code: row.get(3)?,
            kgs_issued: row.get(4)?,
            created_by: row.get(5)?,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
        }))
    }

    pub fn all(conn: &Connection) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare("SELECT * FROM ink_usages ORDER BY created_at DESC")?;
        let ink_usages = stmt.query_map([], |row| Ok(ActualInkUsage {
            id: row.get(0)?,
            shift_id: row.get(1)?,
            colour_id: row.get(2)?,
            batch_code: row.get(3)?,
            kgs_issued: row.get(4)?,
            created_by: row.get(5)?,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
        }))?.collect::<Result<Vec<_>, _>>()?;
        Ok(ink_usages)
    }
    
    pub fn filter(conn: &Connection, filter: &ActualInkUsageFilterPayload) -> Result<FilterResponse<Self>> {
        let mut count_query = "SELECT COUNT(*) FROM ink_usages WHERE 1=1".to_string();
        let mut data_query = "SELECT * FROM ink_usages WHERE 1=1".to_string();
        let mut params_vec: Vec<&dyn rusqlite::ToSql> = vec![];

        let mut shift_ids: Vec<i32> = vec![];
        let mut colour_ids: Vec<i32> = vec![];
        let mut batch_codes: Vec<String> = vec![];
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

        if let Some(val) = &filter.colour_id {
            if let Ok(parsed) = val.parse::<i32>() {
                colour_ids.push(parsed);
                params_vec.push(colour_ids.last().unwrap());
                count_query.push_str(" AND colour_id = ?");
                data_query.push_str(" AND colour_id = ?");
            }
        }

        if let Some(val) = &filter.batch_code {
            if !val.is_empty() {
                batch_codes.push(format!("%{}%", val));
                params_vec.push(batch_codes.last().unwrap());
                count_query.push_str(" AND batch_code LIKE ?");
                data_query.push_str(" AND batch_code LIKE ?");
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
            Ok(ActualInkUsage {
                id: row.get(0)?,
                shift_id: row.get(1)?,
                colour_id: row.get(2)?,
                batch_code: row.get(3)?,
                kgs_issued: row.get(4)?,
                created_by: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?;

        let data = rows.collect::<Result<Vec<_>, _>>()?;

        Ok(FilterResponse {
            total_count,
            data,
        })
    }
}