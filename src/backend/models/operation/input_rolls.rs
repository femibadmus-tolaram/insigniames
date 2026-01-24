#[derive(Deserialize)]
pub struct EndInputRollPayload {
    pub id: i32,
    pub weight_unit: String,
    pub posting_date: String,
    pub batch: String,
    pub input_roll_id: i32,
    pub consumed_weight: String,
    pub material_number: String,
    pub production_order: String,
}
use chrono::Local;
use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};

use crate::sap::post_material_document;

#[derive(Debug, Serialize)]
pub struct InputRoll {
    pub id: i32,
    pub job_id: i32,
    pub batch: String,
    pub material_document: Option<String>,
    pub material_number: Option<String>,
    pub material_description: Option<String>,
    pub process_order: Option<String>,
    pub start_meter: f64,
    pub created_by: i32,
    pub start_weight: String,
    pub consumed_weight: Option<f64>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Deserialize, Clone)]
pub struct InputRollCreatePayload {
    pub job_id: i32,
    pub batch: String,
    pub material_document: Option<String>,
    pub material_number: String,
    pub start_meter: f64,
    pub start_weight: String,
    pub consumed_weight: Option<f64>,
}

#[derive(Deserialize)]
pub struct InputRollUpdatePayload {
    pub id: i32,
    pub batch: Option<String>,
    pub material_document: Option<String>,
    pub material_number: Option<String>,
    pub start_meter: Option<f64>,
    pub start_weight: Option<String>,
    pub consumed_weight: Option<f64>,
}

#[derive(Deserialize)]
pub struct InputRollFilterPayload {
    pub id: Option<i32>,
    pub job_id: Option<i32>,
    pub batch: Option<String>,
    pub material_document: Option<String>,
    pub material_number: Option<String>,
    pub start_meter: Option<f64>,
    pub created_by: Option<i32>,
    pub start_weight: Option<String>,
    pub consumed_weight: Option<f64>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub per_page: Option<i32>,
    pub page: Option<i32>,
}

impl InputRoll {
    pub fn find_by_job_id(conn: &Connection, job_id: i32) -> Result<Self> {
        let mut stmt = conn.prepare(
            "SELECT ir.id, ir.job_id, ir.batch, ir.material_document, ir.material_number, mvd.desc, j.production_order, ir.start_meter, ir.created_by, ir.start_weight, ir.consumed_weight, ir.created_at, ir.updated_at FROM input_rolls ir LEFT JOIN jobs j ON ir.job_id = j.id LEFT JOIN materials_value_description mvd ON ir.material_number = mvd.value WHERE ir.job_id = ?1 LIMIT 1"
        )?;
        stmt.query_row(params![job_id], |row| {
            Ok(InputRoll {
                id: row.get(0)?,
                job_id: row.get(1)?,
                batch: row.get(2)?,
                material_document: row.get(3).ok(),
                material_number: row.get(4).ok(),
                material_description: row.get(5).ok(),
                process_order: row.get(6).ok(),
                start_meter: row.get(7)?,
                created_by: row.get(8)?,
                start_weight: row.get(9)?,
                consumed_weight: row.get(10).ok(),
                created_at: row.get(11)?,
                updated_at: row.get(12)?,
            })
        })
    }

    pub async fn end_input_roll(conn: &Connection, data: &EndInputRollPayload) -> Result<String> {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let now_str = now.as_str();

        let document_number = post_material_document(
            &data.material_number,
            &data.batch,
            &data.production_order,
            &data.consumed_weight,
            &data.weight_unit,
            &data.posting_date,
            "DW01",
        )
        .await
        .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;

        conn.execute(
        "UPDATE input_rolls SET consumed_weight = ?1, updated_at = ?2, material_document = ?3 WHERE id = ?4",
        params![&data.consumed_weight, now_str, &document_number, data.input_roll_id],
    )?;

        Ok(document_number)
    }

    pub fn create(conn: &Connection, data: &InputRollCreatePayload, user_id: i32) -> Result<Self> {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        conn.execute(
            "INSERT INTO input_rolls (job_id, batch, material_document, material_number, start_meter, created_by, start_weight, consumed_weight, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![data.job_id, data.batch, data.material_document.clone().unwrap_or_default(), data.material_number, data.start_meter, user_id, data.start_weight, data.consumed_weight, now, now],
        )?;
        let id = conn.last_insert_rowid() as i32;
        Ok(InputRoll {
            id,
            job_id: data.job_id,
            batch: data.batch.clone(),
            material_document: data.material_document.clone(),
            material_number: Some(data.material_number.clone()),
            material_description: None,
            process_order: None,
            start_meter: data.start_meter,
            created_by: user_id,
            start_weight: data.start_weight.clone(),
            consumed_weight: data.consumed_weight,
            created_at: now.clone(),
            updated_at: now.clone(),
        })
    }

    pub fn update(&mut self, conn: &Connection, data: &InputRollUpdatePayload) -> Result<()> {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        if let Some(batch) = &data.batch {
            conn.execute(
                "UPDATE input_rolls SET batch = ?1 WHERE id = ?2",
                params![batch, self.id],
            )?;
            self.batch = batch.clone();
        }
        if let Some(material_document) = &data.material_document {
            conn.execute(
                "UPDATE input_rolls SET material_document = ?1 WHERE id = ?2",
                params![material_document, self.id],
            )?;
            self.material_document = Some(material_document.clone());
        }
        if let Some(material_number) = &data.material_number {
            conn.execute(
                "UPDATE input_rolls SET material_number = ?1 WHERE id = ?2",
                params![material_number, self.id],
            )?;
            self.material_number = Some(material_number.clone());
        }
        if let Some(start_meter) = data.start_meter {
            conn.execute(
                "UPDATE input_rolls SET start_meter = ?1 WHERE id = ?2",
                params![start_meter, self.id],
            )?;
            self.start_meter = start_meter;
        }
        if let Some(start_weight) = &data.start_weight {
            conn.execute(
                "UPDATE input_rolls SET start_weight = ?1 WHERE id = ?2",
                params![start_weight, self.id],
            )?;
            self.start_weight = start_weight.clone();
        }
        if let Some(consumed_weight) = data.consumed_weight {
            conn.execute(
                "UPDATE input_rolls SET consumed_weight = ?1 WHERE id = ?2",
                params![consumed_weight, self.id],
            )?;
            self.consumed_weight = Some(consumed_weight);
        }
        conn.execute(
            "UPDATE input_rolls SET updated_at = ?1 WHERE id = ?2",
            params![now, self.id],
        )?;
        self.updated_at = now;
        Ok(())
    }

    pub fn delete(&self, conn: &Connection) -> Result<()> {
        conn.execute("DELETE FROM input_rolls WHERE id = ?1", params![self.id])?;
        Ok(())
    }

    pub fn find_by_id(conn: &Connection, id: i32) -> Result<Self> {
        let mut stmt = conn.prepare(
            "SELECT ir.id, ir.job_id, ir.batch, ir.material_document, ir.material_number, mvd.desc, j.production_order, ir.start_meter, ir.created_by, ir.start_weight, ir.consumed_weight, ir.created_at, ir.updated_at FROM input_rolls ir LEFT JOIN jobs j ON ir.job_id = j.id LEFT JOIN materials_value_description mvd ON ir.material_number = mvd.value WHERE ir.id = ?1"
        )?;
        stmt.query_row(params![id], |row| {
            Ok(InputRoll {
                id: row.get(0)?,
                job_id: row.get(1)?,
                batch: row.get(2)?,
                material_document: row.get(3).ok(),
                material_number: row.get(4).ok(),
                material_description: row.get(5).ok(),
                process_order: row.get(6).ok(),
                start_meter: row.get(7)?,
                created_by: row.get(8)?,
                start_weight: row.get(9)?,
                consumed_weight: row.get(10).ok(),
                created_at: row.get(11)?,
                updated_at: row.get(12)?,
            })
        })
    }

    pub fn all(conn: &Connection) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare(
            "SELECT ir.id, ir.job_id, ir.batch, ir.material_document, ir.material_number, mvd.desc, j.production_order, ir.start_meter, ir.created_by, ir.start_weight, ir.consumed_weight, ir.created_at, ir.updated_at FROM input_rolls ir LEFT JOIN jobs j ON ir.job_id = j.id LEFT JOIN materials_value_description mvd ON ir.material_number = mvd.value ORDER BY ir.created_at DESC"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(InputRoll {
                id: row.get(0)?,
                job_id: row.get(1)?,
                batch: row.get(2)?,
                material_document: row.get(3).ok(),
                material_number: row.get(4).ok(),
                material_description: row.get(5).ok(),
                process_order: row.get(6).ok(),
                start_meter: row.get(7)?,
                created_by: row.get(8)?,
                start_weight: row.get(9)?,
                consumed_weight: row.get(10).ok(),
                created_at: row.get(11)?,
                updated_at: row.get(12)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
    }

    pub fn filter(conn: &Connection, filter: &InputRollFilterPayload) -> Result<Vec<Self>> {
        let mut query = String::from(
            "SELECT ir.id, ir.job_id, ir.batch, ir.material_document, ir.material_number, mvd.desc, j.production_order, ir.start_meter, ir.created_by, ir.start_weight, ir.consumed_weight, ir.created_at, ir.updated_at FROM input_rolls ir LEFT JOIN jobs j ON ir.job_id = j.id LEFT JOIN materials_value_description mvd ON ir.material_number = mvd.value WHERE 1=1",
        );
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(id) = filter.id {
            query.push_str(" AND id = ?");
            params_vec.push(Box::new(id));
        }
        if let Some(job_id) = filter.job_id {
            query.push_str(" AND job_id = ?");
            params_vec.push(Box::new(job_id));
        }
        if let Some(batch) = &filter.batch {
            query.push_str(" AND batch LIKE ?");
            params_vec.push(Box::new(format!("%{}%", batch)));
        }
        if let Some(material_document) = &filter.material_document {
            query.push_str(" AND material_document LIKE ?");
            params_vec.push(Box::new(format!("%{}%", material_document)));
        }
        if let Some(material_number) = &filter.material_number {
            query.push_str(" AND material_number LIKE ?");
            params_vec.push(Box::new(format!("%{}%", material_number)));
        }
        if let Some(start_meter) = filter.start_meter {
            query.push_str(" AND start_meter = ?");
            params_vec.push(Box::new(start_meter));
        }
        if let Some(created_by) = filter.created_by {
            query.push_str(" AND created_by = ?");
            params_vec.push(Box::new(created_by));
        }
        if let Some(start_weight) = &filter.start_weight {
            query.push_str(" AND start_weight LIKE ?");
            params_vec.push(Box::new(format!("%{}%", start_weight)));
        }
        if let Some(consumed_weight) = filter.consumed_weight {
            query.push_str(" AND consumed_weight = ?");
            params_vec.push(Box::new(consumed_weight));
        }
        if let Some(created_at) = &filter.created_at {
            query.push_str(" AND created_at LIKE ?");
            params_vec.push(Box::new(format!("%{}%", created_at)));
        }
        if let Some(updated_at) = &filter.updated_at {
            query.push_str(" AND updated_at LIKE ?");
            params_vec.push(Box::new(format!("%{}%", updated_at)));
        }

        query.push_str(" ORDER BY ir.created_at DESC");

        if let (Some(page), Some(per_page)) = (filter.page, filter.per_page) {
            let offset = (page - 1) * per_page;
            query.push_str(" LIMIT ? OFFSET ?");
            params_vec.push(Box::new(per_page));
            params_vec.push(Box::new(offset));
        }

        let mut stmt = conn.prepare(&query)?;
        let rows = stmt.query_map(
            params_vec
                .iter()
                .map(|b| &**b)
                .collect::<Vec<_>>()
                .as_slice(),
            |row| {
                Ok(InputRoll {
                    id: row.get(0)?,
                    job_id: row.get(1)?,
                    batch: row.get(2)?,
                    material_document: row.get(3).ok(),
                    material_number: row.get(4).ok(),
                    material_description: row.get(5).ok(),
                    process_order: row.get(6).ok(),
                    start_meter: row.get(7)?,
                    created_by: row.get(8)?,
                    start_weight: row.get(9)?,
                    consumed_weight: row.get(10).ok(),
                    created_at: row.get(11)?,
                    updated_at: row.get(12)?,
                })
            },
        )?;
        rows.collect::<Result<Vec<_>, _>>()
    }
}
