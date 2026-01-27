use crate::backend::models::{InputRoll, InputRollCreatePayload, InputRollFilterPayload};
use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct Job {
    pub id: i32,
    pub machine_id: i32,
    pub shift_id: i32,
    pub created_by: i32,
    pub production_order: String,
    pub start_datetime: Option<String>,
    pub end_datetime: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Deserialize)]
pub struct JobFilterPayload {
    pub id: Option<i32>,
    pub machine_id: Option<i32>,
    pub shift_id: Option<i32>,
    pub created_by: Option<i32>,
    pub status: Option<String>,
    pub production_order: Option<String>,
    pub start_datetime: Option<String>,
    pub end_datetime: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Deserialize)]
pub struct JobCreatePayload {
    pub machine_id: i32,
    pub shift_id: i32,
    pub production_order: String,
    pub input_roll: InputRollCreatePayload,
}

#[derive(Deserialize)]
pub struct JobUpdatePayload {
    pub id: i32,
    pub machine_id: Option<i32>,
    pub shift_id: Option<i32>,
    pub created_by: Option<i32>,
    pub production_order: Option<String>,
    pub start_datetime: Option<String>,
    pub end_datetime: Option<String>,
}

#[derive(Serialize)]
pub struct JobInputRollMerged {
    pub id: i32,
    pub shift_id: i32,
    pub input_roll_id: i32,
    pub production_order: String,
    pub batch: String,
    pub start_weight: String,
    pub start_meter: Option<f64>,
    pub created_by: i32,
    pub machine_id: i32,
    pub created_at: String,
    pub updated_at: String,
    pub start_datetime: Option<String>,
    pub end_datetime: Option<String>,
    pub consumed_weight: Option<f64>,
    pub material_number: Option<String>,
    pub last_updated: String,
}

#[derive(Serialize)]
pub struct JobInputRollMergedResponse {
    pub total_count: usize,
    pub data: Vec<JobInputRollMerged>,
}

impl Job {
    pub fn create(
        conn: &Connection,
        data: &JobCreatePayload,
        user_id: i32,
    ) -> Result<JobInputRollMerged> {
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        // Check for existing job with same production_order and end_datetime IS NULL (active job)
        let mut stmt = conn.prepare("SELECT id, machine_id, shift_id, created_by, production_order, start_datetime, end_datetime, created_at, updated_at FROM jobs WHERE production_order = ?1 AND end_datetime IS NULL LIMIT 1")?;
        let existing_job = stmt
            .query_row(params![data.production_order.clone()], |row| {
                Ok(Job {
                    id: row.get(0)?,
                    machine_id: row.get(1)?,
                    shift_id: row.get(2)?,
                    created_by: row.get(3)?,
                    production_order: row.get(4)?,
                    start_datetime: row.get(5)?,
                    end_datetime: row.get(6)?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            })
            .ok();

        if let Some(job) = existing_job {
            // Always create a new input roll for the existing job
            let mut input_roll_payload = data.input_roll.clone();
            input_roll_payload.job_id = job.id;
            let input_roll = InputRoll::create(conn, &input_roll_payload, user_id)?;
            return Ok(JobInputRollMerged {
                id: job.id,
                shift_id: job.shift_id,
                input_roll_id: input_roll.id,
                production_order: job.production_order.clone(),
                batch: input_roll.batch.clone(),
                start_weight: input_roll.start_weight.clone(),
                start_meter: Some(input_roll.start_meter),
                created_by: job.created_by,
                machine_id: job.machine_id,
                created_at: job.created_at.clone(),
                updated_at: job.updated_at.clone(),
                start_datetime: job.start_datetime.clone(),
                end_datetime: job.end_datetime.clone(),
                consumed_weight: input_roll.consumed_weight.clone(),
                material_number: input_roll.material_number.clone(),
                last_updated: input_roll.updated_at.clone(),
            });
        }

        // Otherwise, create new job
        conn.execute(
            "INSERT INTO jobs (machine_id, shift_id, created_by, production_order, start_datetime, end_datetime, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                data.machine_id,
                data.shift_id,
                user_id,
                data.production_order,
                Some(now.as_str()), // start_datetime is now
                Option::<String>::None, // end_datetime is None
                now.as_str(),
                now.as_str(),
            ],
        )?;
        let id = conn.last_insert_rowid() as i32;
        let job = Job {
            id,
            machine_id: data.machine_id,
            shift_id: data.shift_id,
            created_by: user_id,
            production_order: data.production_order.clone(),
            start_datetime: Some(now.clone()),
            end_datetime: None,
            created_at: now.clone(),
            updated_at: now,
        };

        let mut input_roll_payload = data.input_roll.clone();
        input_roll_payload.job_id = job.id;
        let input_roll = InputRoll::create(conn, &input_roll_payload, user_id)?;
        Ok(JobInputRollMerged {
            id: job.id,
            shift_id: job.shift_id,
            input_roll_id: input_roll.id,
            production_order: job.production_order.clone(),
            batch: input_roll.batch.clone(),
            start_weight: input_roll.start_weight.clone(),
            start_meter: Some(input_roll.start_meter),
            created_by: job.created_by,
            machine_id: job.machine_id,
            created_at: job.created_at.clone(),
            updated_at: job.updated_at.clone(),
            start_datetime: job.start_datetime.clone(),
            end_datetime: job.end_datetime.clone(),
            consumed_weight: input_roll.consumed_weight.clone(),
            material_number: input_roll.material_number.clone(),
            last_updated: input_roll.updated_at.clone(),
        })
    }

    pub fn filter_with_input_rolls(
        conn: &Connection,
        filter: &JobFilterPayload,
    ) -> Result<JobInputRollMergedResponse> {
        let status_is_active = filter
            .status
            .as_deref()
            .map(|s| s.eq_ignore_ascii_case("active"))
            .unwrap_or(false);
        let mut sql = String::from(
            "SELECT id, machine_id, shift_id, created_by, production_order, start_datetime, end_datetime, created_at, updated_at FROM jobs WHERE 1=1",
        );
        let mut params: Vec<rusqlite::types::Value> = Vec::new();
        if let Some(id) = filter.id {
            sql.push_str(" AND id = ?");
            params.push(id.into());
        }
        if let Some(machine_id) = filter.machine_id {
            sql.push_str(" AND machine_id = ?");
            params.push(machine_id.into());
        }
        if let Some(shift_id) = filter.shift_id {
            sql.push_str(" AND shift_id = ?");
            params.push(shift_id.into());
        }
        if let Some(created_by) = filter.created_by {
            sql.push_str(" AND created_by = ?");
            params.push(created_by.into());
        }
        if let Some(ref production_order) = filter.production_order {
            sql.push_str(" AND production_order = ?");
            params.push(production_order.clone().into());
        }
        if let Some(ref start_datetime) = filter.start_datetime {
            sql.push_str(" AND start_datetime = ?");
            params.push(start_datetime.clone().into());
        }
        if let Some(ref end_datetime) = filter.end_datetime {
            sql.push_str(" AND end_datetime = ?");
            params.push(end_datetime.clone().into());
        }
        if let Some(ref created_at) = filter.created_at {
            sql.push_str(" AND created_at = ?");
            params.push(created_at.clone().into());
        }
        if let Some(ref updated_at) = filter.updated_at {
            sql.push_str(" AND updated_at = ?");
            params.push(updated_at.clone().into());
        }
        sql.push_str(" ORDER BY created_at DESC");

        let mut stmt = conn.prepare(&sql)?;
        let jobs = stmt.query_map(rusqlite::params_from_iter(params.iter()), |row| {
            Ok(Job {
                id: row.get(0)?,
                machine_id: row.get(1)?,
                shift_id: row.get(2)?,
                created_by: row.get(3)?,
                production_order: row.get(4)?,
                start_datetime: row.get(5)?,
                end_datetime: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })?;

        let mut merged = Vec::new();
        for job in jobs {
            let job = job?;
            let input_filter = InputRollFilterPayload {
                id: None,
                job_id: Some(job.id),
                batch: None,
                material_document: None,
                material_number: None,
                start_meter: None,
                created_by: None,
                start_weight: None,
                consumed_weight: None,
                created_at: None,
                updated_at: None,
                per_page: None,
                page: None,
            };
            let input_rolls: Vec<InputRoll> = if status_is_active {
                InputRoll::filter(conn, &input_filter)?
                    .into_iter()
                    .filter(|r| r.consumed_weight.is_none())
                    .collect()
            } else {
                InputRoll::filter(conn, &input_filter)?
            };
            for input_roll in input_rolls {
                merged.push(JobInputRollMerged {
                    id: job.id,
                    shift_id: job.shift_id,
                    input_roll_id: input_roll.id,
                    production_order: job.production_order.clone(),
                    batch: input_roll.batch.clone(),
                    start_weight: input_roll.start_weight.clone(),
                    start_meter: Some(input_roll.start_meter),
                    created_by: job.created_by,
                    machine_id: job.machine_id,
                    created_at: job.created_at.clone(),
                    updated_at: job.updated_at.clone(),
                    start_datetime: job.start_datetime.clone(),
                    end_datetime: job.end_datetime.clone(),
                    consumed_weight: input_roll.consumed_weight.clone(),
                    material_number: input_roll.material_number.clone(),
                    last_updated: input_roll.updated_at.clone(),
                });
            }
        }
        Ok(JobInputRollMergedResponse {
            total_count: merged.len(),
            data: merged,
        })
    }

    pub fn find_by_id(conn: &Connection, id: i32) -> Result<Self> {
        let mut stmt = conn.prepare(
            "SELECT id, machine_id, shift_id, created_by, production_order, start_datetime, end_datetime, created_at, updated_at FROM jobs WHERE id = ?1"
        )?;
        stmt.query_row(params![id], |row| {
            Ok(Job {
                id: row.get(0)?,
                machine_id: row.get(1)?,
                shift_id: row.get(2)?,
                created_by: row.get(3)?,
                production_order: row.get(4)?,
                start_datetime: row.get(5)?,
                end_datetime: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })
    }

    pub fn all(conn: &Connection) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare(
            "SELECT id, machine_id, shift_id, created_by, production_order, start_datetime, end_datetime, created_at, updated_at FROM jobs ORDER BY created_at DESC"
        )?;
        let jobs = stmt
            .query_map([], |row| {
                Ok(Job {
                    id: row.get(0)?,
                    machine_id: row.get(1)?,
                    shift_id: row.get(2)?,
                    created_by: row.get(3)?,
                    production_order: row.get(4)?,
                    start_datetime: row.get(5)?,
                    end_datetime: row.get(6)?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(jobs)
    }

    pub fn update(&mut self, conn: &Connection, data: &JobUpdatePayload) -> Result<()> {
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        if let Some(machine_id) = data.machine_id {
            conn.execute(
                "UPDATE jobs SET machine_id = ?1 WHERE id = ?2",
                params![machine_id, self.id],
            )?;
            self.machine_id = machine_id;
        }
        if let Some(shift_id) = data.shift_id {
            conn.execute(
                "UPDATE jobs SET shift_id = ?1 WHERE id = ?2",
                params![shift_id, self.id],
            )?;
            self.shift_id = shift_id;
        }
        if let Some(created_by) = data.created_by {
            conn.execute(
                "UPDATE jobs SET created_by = ?1 WHERE id = ?2",
                params![created_by, self.id],
            )?;
            self.created_by = created_by;
        }
        if let Some(ref production_order) = data.production_order {
            conn.execute(
                "UPDATE jobs SET production_order = ?1 WHERE id = ?2",
                params![production_order, self.id],
            )?;
            self.production_order = production_order.clone();
        }
        if let Some(ref start_datetime) = data.start_datetime {
            conn.execute(
                "UPDATE jobs SET start_datetime = ?1 WHERE id = ?2",
                params![start_datetime, self.id],
            )?;
            self.start_datetime = Some(start_datetime.clone());
        }
        if let Some(ref end_datetime) = data.end_datetime {
            conn.execute(
                "UPDATE jobs SET end_datetime = ?1 WHERE id = ?2",
                params![end_datetime, self.id],
            )?;
            self.end_datetime = Some(end_datetime.clone());
        }
        conn.execute(
            "UPDATE jobs SET updated_at = ?1 WHERE id = ?2",
            params![now, self.id],
        )?;
        self.updated_at = now;
        Ok(())
    }

    pub fn delete(&self, conn: &mut Connection) -> Result<()> {
        let has_output_rolls: i32 = conn.query_row(
            "SELECT COUNT(*) FROM output_rolls o \
             JOIN input_rolls i ON o.input_roll_id = i.id \
             WHERE i.job_id = ?1",
            params![self.id],
            |row| row.get(0),
        )?;

        if has_output_rolls > 0 {
            return Err(rusqlite::Error::InvalidParameterName(
                "Cannot delete job: output rolls exist for this job".to_string(),
            ));
        }

        let tx = conn.transaction()?;
        tx.execute(
            "DELETE FROM input_rolls WHERE job_id = ?1",
            params![self.id],
        )?;
        tx.execute("DELETE FROM jobs WHERE id = ?1", params![self.id])?;
        tx.commit()?;
        Ok(())
    }
}
