use chrono::Local;
use rusqlite::{params, Connection, Result};
use serde::{Serialize, Deserialize};
use crate::backend::models::FilterResponse;

#[derive(Debug, Serialize)]
pub struct Scrap {
    pub id: i32,
    pub shift_id: i32,
    pub time: String,
    pub scrap_type_id: i32,
    pub weight_kg: f64,
    pub notes: Option<String>,
    pub created_by: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Deserialize)]
pub struct ScrapCreatePayload {
    pub shift_id: i32,
    pub time: String,
    pub scrap_type_id: i32,
    pub weight_kg: f64,
    pub notes: Option<String>,
}

#[derive(Deserialize)]
pub struct ScrapPayload {
    pub id: i32,
    pub shift_id: Option<i32>,
    pub time: Option<String>,
    pub scrap_type_id: Option<i32>,
    pub weight_kg: Option<f64>,
    pub notes: Option<String>,
}

#[derive(Deserialize)]
pub struct ScrapFilterPayload {
    pub shift_id: Option<String>,
    pub scrap_type_id: Option<String>,
    pub created_by: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub per_page: Option<String>,
    pub page: Option<String>,
}

impl Scrap {
    pub fn create(conn: &Connection, data: &ScrapCreatePayload, user_id: i32) -> Result<Self> {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        conn.execute(
            "INSERT INTO scraps (shift_id, time, scrap_type_id, weight_kg, notes, created_by, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![data.shift_id, data.time, data.scrap_type_id, data.weight_kg, data.notes, user_id, now, now],
        )?;
        let id = conn.last_insert_rowid() as i32;
        Ok(Scrap {
            id,
            shift_id: data.shift_id,
            time: data.time.clone(),
            scrap_type_id: data.scrap_type_id,
            weight_kg: data.weight_kg,
            notes: data.notes.clone(),
            created_by: user_id,
            created_at: now.clone(),
            updated_at: now.clone(),
        })
    }

    pub fn update(&mut self, conn: &Connection, data: &ScrapPayload) -> Result<()> {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        if let Some(shift_id) = data.shift_id {
            conn.execute("UPDATE scraps SET shift_id = ?1 WHERE id = ?2", params![shift_id, self.id])?;
            self.shift_id = shift_id;
        }
        if let Some(time) = &data.time {
            conn.execute("UPDATE scraps SET time = ?1 WHERE id = ?2", params![time, self.id])?;
            self.time = time.clone();
        }
        if let Some(scrap_type_id) = data.scrap_type_id {
            conn.execute("UPDATE scraps SET scrap_type_id = ?1 WHERE id = ?2", params![scrap_type_id, self.id])?;
            self.scrap_type_id = scrap_type_id;
        }
        if let Some(weight_kg) = data.weight_kg {
            conn.execute("UPDATE scraps SET weight_kg = ?1 WHERE id = ?2", params![weight_kg, self.id])?;
            self.weight_kg = weight_kg;
        }
        if let Some(notes) = &data.notes {
            conn.execute("UPDATE scraps SET notes = ?1 WHERE id = ?2", params![notes, self.id])?;
            self.notes = Some(notes.clone());
        }
        conn.execute("UPDATE scraps SET updated_at = ?1 WHERE id = ?2", params![now, self.id])?;
        self.updated_at = now;
        Ok(())
    }

    pub fn delete(&self, conn: &Connection) -> Result<()> {
        conn.execute("DELETE FROM scraps WHERE id = ?1", params![self.id])?;
        Ok(())
    }

    pub fn find_by_id(conn: &Connection, id: i32) -> Result<Self> {
        let mut stmt = conn.prepare("SELECT * FROM scraps WHERE id = ?1")?;
        stmt.query_row(params![id], |row| Ok(Scrap {
            id: row.get(0)?,
            shift_id: row.get(1)?,
            time: row.get(2)?,
            scrap_type_id: row.get(3)?,
            weight_kg: row.get(4)?,
            notes: row.get(5)?,
            created_by: row.get(6)?,
            created_at: row.get(7)?,
            updated_at: row.get(8)?,
        }))
    }

    pub fn all(conn: &Connection) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare("SELECT * FROM scraps ORDER BY time DESC")?;
        let scraps = stmt.query_map([], |row| Ok(Scrap {
            id: row.get(0)?,
            shift_id: row.get(1)?,
            time: row.get(2)?,
            scrap_type_id: row.get(3)?,
            weight_kg: row.get(4)?,
            notes: row.get(5)?,
            created_by: row.get(6)?,
            created_at: row.get(7)?,
            updated_at: row.get(8)?,
        }))?.collect::<Result<Vec<_>, _>>()?;
        Ok(scraps)
    }

    pub fn filter(conn: &Connection, filter: &ScrapFilterPayload) -> Result<FilterResponse<Self>> {
        let mut count_query = "SELECT COUNT(*) FROM scraps WHERE 1=1".to_string();
        let mut data_query = "SELECT * FROM scraps WHERE 1=1".to_string();
        let mut params_vec: Vec<&dyn rusqlite::ToSql> = vec![];

        let mut shift_ids: Vec<i32> = vec![];
        let mut scrap_type_ids: Vec<i32> = vec![];
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

        if let Some(val) = &filter.scrap_type_id {
            if let Ok(parsed) = val.parse::<i32>() {
                scrap_type_ids.push(parsed);
                params_vec.push(scrap_type_ids.last().unwrap());
                count_query.push_str(" AND scrap_type_id = ?");
                data_query.push_str(" AND scrap_type_id = ?");
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
                count_query.push_str(" AND date(time) >= date(?)");
                data_query.push_str(" AND date(time) >= date(?)");
            }
        }

        if let Some(val) = &filter.end_date {
            if !val.is_empty() {
                end_dates.push(val.clone());
                params_vec.push(end_dates.last().unwrap());
                count_query.push_str(" AND date(time) <= date(?)");
                data_query.push_str(" AND date(time) <= date(?)");
            }
        }

        let total_count: i32 = conn.query_row(&count_query, params_vec.as_slice(), |row| row.get(0))?;

        data_query.push_str(" ORDER BY time DESC");

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
            Ok(Scrap {
                id: row.get(0)?,
                shift_id: row.get(1)?,
                time: row.get(2)?,
                scrap_type_id: row.get(3)?,
                weight_kg: row.get(4)?,
                notes: row.get(5)?,
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

