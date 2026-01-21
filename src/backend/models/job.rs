use crate::backend::models::FilterResponse;
use crate::sap::post_material_document;
use chrono::Local;
use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct Job {
    pub id: i32,
    pub shift_id: i32,
    pub production_order: String,
    pub batch_roll_no: String,
    pub start_weight: String,
    pub start_meter: f64,
    pub created_by: i32,
    pub machine_id: i32,
    pub created_at: String,
    pub updated_at: String,
    pub start_datetime: String,
    pub end_datetime: Option<String>,
    pub consumed_weight: Option<String>,
    pub material_number: Option<String>,
    pub material_description: Option<String>,
}

#[derive(Deserialize)]
pub struct JobCreatePayload {
    pub shift_id: i32,
    pub production_order: String,
    pub batch_roll_no: String,
    pub start_weight: String,
    pub start_meter: f64,
    pub machine_id: i32,
    pub material_number: String,
}

#[derive(Debug, Serialize)]
pub struct JobWithStats {
    pub id: i32,
    pub shift_id: i32,
    pub production_order: String,
    pub batch_roll_no: String,
    pub start_weight: String,
    pub start_meter: f64,
    pub created_by: i32,
    pub machine_id: i32,
    pub created_at: String,
    pub updated_at: String,
    pub start_datetime: String,
    pub end_datetime: Option<String>,
    pub consumed_weight: Option<String>,
    pub material_number: Option<String>,
    pub material_description: Option<String>,
    pub total_rolls: i32,
    pub pending_rolls: i32,
    pub total_weight: f64,
    pub last_updated: String,
}

#[derive(Deserialize)]
pub struct JobPayload {
    pub id: i32,
    pub shift_id: Option<i32>,
    pub production_order: Option<String>,
    pub batch_roll_no: Option<String>,
    pub start_weight: Option<String>,
    pub start_meter: Option<f64>,
    pub machine_id: Option<i32>,
    pub end_datetime: Option<String>,
    pub consumed_weight: Option<String>,
    pub material_number: Option<String>,
}

#[derive(Deserialize)]
pub struct JobFilterPayload {
    pub shift_id: Option<String>,
    pub production_order: Option<String>,
    pub batch_roll_no: Option<String>,
    pub last_insert: Option<String>,
    pub created_by: Option<String>,
    pub machine_id: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub per_page: Option<String>,
    pub status: Option<String>,
    pub page: Option<String>,
}

#[derive(Deserialize)]
pub struct EndJobPayload {
    pub id: i32,
    pub weight_unit: String,
    pub posting_date: String,
    pub batch_roll_no: String,
    pub consumed_weight: String,
    pub material_number: String,
    pub production_order: String,
}

fn get_material_description(conn: &Connection, material_number: &str) -> Option<String> {
    conn.query_row(
        "SELECT desc FROM materials_value_description WHERE value = ?",
        params![material_number],
        |row| row.get::<_, String>(0),
    )
    .ok()
}

impl Job {
    pub async fn end_job(conn: &Connection, data: &EndJobPayload) -> Result<String> {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let now_str = now.as_str();

        let document_number = post_material_document(
            &data.material_number,
            &data.batch_roll_no,
            &data.production_order,
            &data.consumed_weight,
            &data.weight_unit,
            &data.posting_date,
            "DW01",
        )
        .await
        .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;

        conn.execute(
        "UPDATE jobs SET end_datetime = ?1, consumed_weight = ?2, updated_at = ?3, material_document = ?4 WHERE id = ?5",
        params![now_str, &data.consumed_weight, now_str, &document_number, data.id],
        // conn.execute(
        // "UPDATE jobs SET consumed_weight = ?1, updated_at = ?2, material_document = ?3 WHERE id = ?4",
        // params![&data.consumed_weight, now_str, &document_number, data.id],
    )?;

        Ok(document_number)
    }

    pub fn has_rolls(conn: &Connection, job_id: i32) -> Result<bool> {
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM rolls WHERE job_id = ?1")?;
        let roll_count: i32 = stmt.query_row(params![job_id], |row| row.get(0))?;
        Ok(roll_count > 0)
    }

    pub fn create(conn: &Connection, data: &JobCreatePayload, user_id: i32) -> Result<Self> {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let material_description = get_material_description(conn, &data.material_number);

        conn.execute(
            "INSERT INTO jobs (shift_id, production_order, batch_roll_no, start_weight, start_meter, created_by, machine_id, created_at, updated_at, start_datetime, material_number) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![data.shift_id, data.production_order, data.batch_roll_no, data.start_weight, data.start_meter, user_id, data.machine_id, now, now, now, &data.material_number],
        )?;
        let id = conn.last_insert_rowid() as i32;
        Ok(Job {
            id,
            shift_id: data.shift_id,
            production_order: data.production_order.clone(),
            batch_roll_no: data.batch_roll_no.clone(),
            start_weight: data.start_weight.clone(),
            start_meter: data.start_meter,
            created_by: user_id,
            machine_id: data.machine_id,
            created_at: now.clone(),
            updated_at: now.clone(),
            start_datetime: now.clone(),
            end_datetime: None,
            consumed_weight: None,
            material_number: Some(data.material_number.clone()),
            material_description,
        })
    }

    pub fn update(&mut self, conn: &Connection, data: &JobPayload) -> Result<()> {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        if let Some(shift_id) = data.shift_id {
            conn.execute(
                "UPDATE jobs SET shift_id = ?1 WHERE id = ?2",
                params![shift_id, self.id],
            )?;
            self.shift_id = shift_id;
        }
        if let Some(production_order) = &data.production_order {
            conn.execute(
                "UPDATE jobs SET production_order = ?1 WHERE id = ?2",
                params![production_order, self.id],
            )?;
            self.production_order = production_order.clone();
        }
        if let Some(batch_roll_no) = &data.batch_roll_no {
            conn.execute(
                "UPDATE jobs SET batch_roll_no = ?1 WHERE id = ?2",
                params![batch_roll_no, self.id],
            )?;
            self.batch_roll_no = batch_roll_no.clone();
        }
        if let Some(start_weight) = data.start_weight.clone() {
            conn.execute(
                "UPDATE jobs SET start_weight = ?1 WHERE id = ?2",
                params![start_weight, self.id],
            )?;
            self.start_weight = start_weight;
        }
        if let Some(start_meter) = data.start_meter {
            conn.execute(
                "UPDATE jobs SET start_meter = ?1 WHERE id = ?2",
                params![start_meter, self.id],
            )?;
            self.start_meter = start_meter;
        }
        if let Some(machine_id) = data.machine_id {
            conn.execute(
                "UPDATE jobs SET machine_id = ?1 WHERE id = ?2",
                params![machine_id, self.id],
            )?;
            self.machine_id = machine_id;
        }
        if let Some(end_datetime) = &data.end_datetime {
            conn.execute(
                "UPDATE jobs SET end_datetime = ?1 WHERE id = ?2",
                params![end_datetime, self.id],
            )?;
            self.end_datetime = Some(end_datetime.clone());
        }
        if let Some(consumed_weight) = &data.consumed_weight {
            conn.execute(
                "UPDATE jobs SET consumed_weight = ?1 WHERE id = ?2",
                params![consumed_weight, self.id],
            )?;
            self.consumed_weight = Some(consumed_weight.clone());
        }
        if let Some(material_number) = &data.material_number {
            conn.execute(
                "UPDATE jobs SET material_number = ?1 WHERE id = ?2",
                params![material_number, self.id],
            )?;
            self.material_number = Some(material_number.clone());
            self.material_description = get_material_description(conn, material_number);
        }
        conn.execute(
            "UPDATE jobs SET updated_at = ?1 WHERE id = ?2",
            params![now, self.id],
        )?;
        self.updated_at = now;
        Ok(())
    }

    pub fn delete(&self, conn: &Connection) -> Result<()> {
        conn.execute("DELETE FROM jobs WHERE id = ?1", params![self.id])?;
        Ok(())
    }

    pub fn find_by_id(conn: &Connection, id: i32) -> Result<Self> {
        let mut stmt = conn.prepare(
        "SELECT id, shift_id, production_order, batch_roll_no, start_weight, start_meter, created_by, machine_id, created_at, updated_at, start_datetime, end_datetime, consumed_weight, material_number FROM jobs WHERE id = ?1"
    )?;
        stmt.query_row(params![id], |row| {
            let material_number: Option<String> = row.get(13)?;
            let material_description = material_number
                .as_ref()
                .and_then(|num| get_material_description(conn, num));

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
                start_datetime: row.get(10)?,
                end_datetime: row.get(11)?,
                consumed_weight: row.get(12)?,
                material_number,
                material_description,
            })
        })
    }

    pub fn all(conn: &Connection) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare(
        "SELECT id, shift_id, production_order, batch_roll_no, start_weight, start_meter, created_by, machine_id, created_at, updated_at, start_datetime, end_datetime, consumed_weight, material_number FROM jobs ORDER BY created_at DESC"
    )?;
        let rows = stmt.query_map([], |row| {
            let material_number: Option<String> = row.get(13)?;
            let material_description = material_number
                .as_ref()
                .and_then(|num| get_material_description(conn, num));

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
                start_datetime: row.get(10)?,
                end_datetime: row.get(11)?,
                consumed_weight: row.get(12)?,
                material_number,
                material_description,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
    }

    pub fn filter(
        conn: &Connection,
        filter: &JobFilterPayload,
    ) -> Result<FilterResponse<JobWithStats>> {
        let mut count_query = "SELECT COUNT(*) FROM jobs j WHERE 1=1".to_string();
        let mut data_query = "
        SELECT 
            j.id, j.shift_id, j.production_order, j.batch_roll_no, j.start_weight, j.start_meter, j.created_by, j.machine_id, j.created_at, j.updated_at, j.start_datetime, j.end_datetime, j.consumed_weight, j.material_number,
            COUNT(r.id) as total_rolls,
            SUM(CASE WHEN r.final_weight = 0 THEN 1 ELSE 0 END) as pending_rolls,
            COALESCE(SUM(r.final_weight), 0) as total_weight,
            COALESCE(MAX(r.updated_at), j.updated_at) as last_updated
        FROM jobs j 
        LEFT JOIN rolls r ON j.id = r.job_id 
        WHERE 1=1
    "
    .to_string();

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
                count_query.push_str(" AND j.shift_id = ?");
                data_query.push_str(" AND j.shift_id = ?");
            }
        }

        if let Some(val) = &filter.production_order {
            if !val.is_empty() {
                production_orders.push(format!("%{}%", val));
                params_vec.push(production_orders.last().unwrap());
                count_query.push_str(" AND j.production_order LIKE ?");
                data_query.push_str(" AND j.production_order LIKE ?");
            }
        }

        if let Some(val) = &filter.batch_roll_no {
            if !val.is_empty() {
                batch_roll_nos.push(format!("%{}%", val));
                params_vec.push(batch_roll_nos.last().unwrap());
                count_query.push_str(" AND j.batch_roll_no LIKE ?");
                data_query.push_str(" AND j.batch_roll_no LIKE ?");
            }
        }

        if let Some(val) = &filter.created_by {
            if let Ok(parsed) = val.parse::<i32>() {
                created_bys.push(parsed);
                params_vec.push(created_bys.last().unwrap());
                count_query.push_str(" AND j.created_by = ?");
                data_query.push_str(" AND j.created_by = ?");
            }
        }

        if let Some(val) = &filter.machine_id {
            if let Ok(parsed) = val.parse::<i32>() {
                machine_ids.push(parsed);
                params_vec.push(machine_ids.last().unwrap());
                count_query.push_str(" AND j.machine_id = ?");
                data_query.push_str(" AND j.machine_id = ?");
            }
        }

        if let Some(status) = &filter.status {
            if status == "active" {
                count_query.push_str(" AND j.end_datetime IS NULL");
                data_query.push_str(" AND j.end_datetime IS NULL");
            } else if status == "completed" {
                count_query.push_str(" AND j.end_datetime IS NOT NULL");
                data_query.push_str(" AND j.end_datetime IS NOT NULL");
            }
        }

        if let Some(val) = &filter.start_date {
            if !val.is_empty() {
                start_dates.push(val.clone());
                params_vec.push(start_dates.last().unwrap());
                count_query.push_str(" AND date(j.created_at) >= date(?)");
                data_query.push_str(" AND date(j.created_at) >= date(?)");
            }
        }

        if let Some(val) = &filter.end_date {
            if !val.is_empty() {
                end_dates.push(val.clone());
                params_vec.push(end_dates.last().unwrap());
                count_query.push_str(" AND date(j.created_at) <= date(?)");
                data_query.push_str(" AND date(j.created_at) <= date(?)");
            }
        }

        data_query.push_str(" GROUP BY j.id");

        let total_count: i32 =
            conn.query_row(&count_query, params_vec.as_slice(), |row| row.get(0))?;

        data_query.push_str(" ORDER BY j.created_at DESC");

        if let (Some(page), Some(per_page)) = (&filter.page, &filter.per_page) {
            if let (Ok(page_val), Ok(per_page_val)) = (page.parse::<i32>(), per_page.parse::<i32>())
            {
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

        if let Some(last_insert) = &filter.last_insert {
            if last_insert == "true" {
                // Only return the single latest job (highest id)
                data_query = data_query.replace(
                    " GROUP BY j.id ORDER BY j.created_at DESC",
                    " GROUP BY j.id ORDER BY j.id DESC LIMIT 1",
                );
            }
        }

        let mut stmt = conn.prepare(&data_query)?;
        let rows = stmt.query_map(params_vec.as_slice(), |row| {
            let material_number: Option<String> = row.get(13)?;
            let material_description = material_number
                .as_ref()
                .and_then(|num| get_material_description(conn, num));

            Ok(JobWithStats {
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
                start_datetime: row.get(10)?,
                end_datetime: row.get(11)?,
                consumed_weight: row.get(12)?,
                material_number,
                material_description,
                total_rolls: row.get(14)?,
                pending_rolls: row.get(15)?,
                total_weight: row.get(16)?,
                last_updated: row.get(17)?,
            })
        })?;

        let data = rows.collect::<Result<Vec<_>, _>>()?;

        Ok(FilterResponse { total_count, data })
    }
}
