use std::io::{Error, ErrorKind};

use crate::{
    backend::models::{FilterResponse, Job},
    sap::{RollData, post_rolls},
};
use chrono::{Datelike, Local};
use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct Roll {
    pub id: i32,
    pub output_roll_no: String,
    pub final_meter: f64,
    pub flag_reason: Option<String>,
    pub final_weight: f64,
    pub core_weight: Option<f64>,
    pub job_id: i32,
    pub created_by: i32,
    pub created_at: String,
    pub updated_at: String,
    pub from_batch: String,
    pub flag_count: i32,
}

#[derive(Deserialize)]
pub struct RollCreatePayload {
    pub final_meter: f64,
    pub flag_reason: Option<String>,
    pub final_weight: f64,
    pub core_weight: Option<f64>,
    pub job_id: i32,
    pub flag_count: i32,
}

#[derive(Deserialize)]
pub struct RollPayload {
    pub id: i32,
    pub output_roll_no: Option<String>,
    pub final_meter: Option<f64>,
    pub flag_reason: Option<String>,
    pub final_weight: Option<f64>,
    pub core_weight: Option<f64>,
    pub job_id: Option<i32>,
}

#[derive(Deserialize)]
pub struct RollFilterPayload {
    pub job_id: Option<String>,
    pub shift_id: Option<String>,
    pub output_roll_no: Option<String>,
    pub flag_reason: Option<String>,
    pub created_by: Option<String>,
    pub section_ids: Option<Vec<String>>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub per_page: Option<String>,
    pub page: Option<String>,
    pub status: Option<String>,
    pub production_order: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RollDetails {
    pub material_number: String,
    pub material_description: String,
    pub process_order_description: String,
    pub production_order: String,
    pub output_roll_no: String,
    pub final_weight: f64,
    pub final_meter: f64,
    pub flag_reason: Option<String>,
    pub created_at: String,
    pub section: String,
    pub flag_count: i32,
}

impl Roll {
    pub fn create(conn: &Connection, data: &RollCreatePayload, user_id: i32) -> Result<Self> {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let job = Job::find_by_id(conn, data.job_id)?;

        let machine = conn.query_row(
            "SELECT label FROM machines WHERE id = ?1",
            params![job.machine_id],
            |row| row.get::<_, String>(0),
        )?;

        let current_date = Local::now();
        let year = current_date.format("%y").to_string();
        let day_of_year = current_date.date_naive().ordinal() as i32;
        let shift_number = (day_of_year - 1) * 2 + job.shift_id;
        let today_start = current_date.format("%Y-%m-%d").to_string();

        let process_order = &job.production_order;

        // Get all jobs for this production_order, order by id asc
        let mut stmt = conn.prepare(
            "SELECT id, batch_roll_no FROM jobs WHERE production_order = ? ORDER BY id ASC",
        )?;
        let jobs: Vec<(i32, String)> = stmt
            .query_map(params![process_order], |row| Ok((row.get(0)?, row.get(1)?)))?
            .filter_map(|r| r.ok())
            .collect();

        // Find jobs with at least one roll
        let mut jobs_with_rolls = Vec::new();
        let mut jobs_without_rolls = Vec::new();
        for (job_id, batch_roll_no) in &jobs {
            let count: i32 = conn
                .query_row(
                    "SELECT COUNT(*) FROM rolls WHERE job_id = ?1",
                    params![job_id],
                    |row| row.get(0),
                )
                .unwrap_or(0);
            if count > 0 {
                jobs_with_rolls.push(batch_roll_no.clone());
            } else {
                jobs_without_rolls.push(batch_roll_no.clone());
            }
        }

        // Merge all previous scenarios and add the new priority condition
        let from_batch = if !jobs_with_rolls.is_empty() {
            let last_with_roll = jobs_with_rolls.last().unwrap();
            if *last_with_roll != job.batch_roll_no {
                // Last job with roll is not the current job, join their batch_roll_no and the current job's batch_roll_no
                format!("{},{}", last_with_roll, job.batch_roll_no)
            } else if jobs_without_rolls.is_empty() {
                // Only jobs with rolls, take the last one's batch_roll_no
                last_with_roll.clone()
            } else {
                // Both exist, join last with rolls and all without rolls
                let mut batches = Vec::new();
                batches.push(last_with_roll.clone());
                batches.extend(jobs_without_rolls.iter().cloned());
                batches.join(",")
            }
        } else if !jobs_without_rolls.is_empty() {
            // Only jobs without rolls, join all batch_roll_no with ","
            jobs_without_rolls.join(",")
        } else {
            job.batch_roll_no.clone()
        };

        let roll_count_for_process_order: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM rolls r JOIN jobs j ON r.job_id = j.id WHERE j.production_order = ?1 AND date(r.created_at) = date(?2)",
                params![process_order, today_start],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let roll_number = roll_count_for_process_order + 1;
        let output_roll_no = format!("{}{:03}{}{:03}", year, shift_number, machine, roll_number);

        conn.execute(
            "INSERT INTO rolls (output_roll_no, final_meter, flag_reason, final_weight, core_weight, job_id, created_by, created_at, updated_at, from_batch, flag_count) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![output_roll_no, data.final_meter, data.flag_reason, data.final_weight, data.core_weight, data.job_id, user_id, now, now, from_batch, data.flag_count],
        )?;

        let id = conn.last_insert_rowid() as i32;
        Ok(Roll {
            id,
            output_roll_no,
            final_meter: data.final_meter,
            flag_reason: data.flag_reason.clone(),
            final_weight: data.final_weight,
            core_weight: data.core_weight,
            job_id: data.job_id,
            created_by: user_id,
            created_at: now.clone(),
            updated_at: now.clone(),
            from_batch,
            flag_count: data.flag_count,
        })
    }

    pub async fn update(&mut self, conn: &Connection, data: &RollPayload) -> Result<()> {
        if let Some(final_weight) = data.final_weight {
            let job = Job::find_by_id(conn, self.job_id)?;

            // Find core_weight from DB (self.core_weight)
            let core_weight = self.core_weight.unwrap_or(0.0);
            let net_weight = if final_weight > core_weight {
                final_weight - core_weight
            } else {
                0.0
            };

            let kg = net_weight;
            let meter = self.final_meter;
            let ratio = if kg > 0.0 {
                (meter / kg * 100.0).round() / 100.0
            } else {
                0.0
            };
            let new_alternate_quantity = (ratio * kg * 100.0).round() / 100.0;

            let roll_data = RollData {
                meter: new_alternate_quantity.to_string(),
                weight: net_weight.to_string(),
                batch: self.output_roll_no.to_string(),
                production_order: job.production_order.clone(),
            };

            if let Err(e) = post_rolls(roll_data).await {
                return Err(rusqlite::Error::ToSqlConversionFailure(Box::new(
                    Error::new(ErrorKind::Other, e),
                )));
            }

            conn.execute(
                "UPDATE rolls SET final_meter = ?1, final_weight = ?2 WHERE id = ?3",
                params![new_alternate_quantity, net_weight, self.id],
            )?;
            self.final_meter = new_alternate_quantity;
            self.final_weight = net_weight;
        }

        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        if let Some(output_roll_no) = &data.output_roll_no {
            conn.execute(
                "UPDATE rolls SET output_roll_no = ?1 WHERE id = ?2",
                params![output_roll_no, self.id],
            )?;
            self.output_roll_no = output_roll_no.clone();
        }
        if let Some(final_meter) = data.final_meter {
            conn.execute(
                "UPDATE rolls SET final_meter = ?1 WHERE id = ?2",
                params![final_meter, self.id],
            )?;
            self.final_meter = final_meter;
        }
        if let Some(flag_reason) = &data.flag_reason {
            conn.execute(
                "UPDATE rolls SET flag_reason = ?1 WHERE id = ?2",
                params![flag_reason, self.id],
            )?;
            self.flag_reason = Some(flag_reason.to_string());
        }
        if let Some(final_weight) = data.final_weight {
            conn.execute(
                "UPDATE rolls SET final_weight = ?1 WHERE id = ?2",
                params![final_weight, self.id],
            )?;
            self.final_weight = final_weight;
        }
        if let Some(job_id) = data.job_id {
            let job = Job::find_by_id(conn, job_id)?;
            conn.execute(
                "UPDATE rolls SET job_id = ?1, from_batch = ?2 WHERE id = ?3",
                params![job_id, job.batch_roll_no, self.id],
            )?;
            self.job_id = job_id;
            self.from_batch = job.batch_roll_no;
        }
        conn.execute(
            "UPDATE rolls SET updated_at = ?1 WHERE id = ?2",
            params![now, self.id],
        )?;
        self.updated_at = now;
        Ok(())
    }

    pub fn delete(&self, conn: &Connection) -> Result<()> {
        conn.execute("DELETE FROM rolls WHERE id = ?1", params![self.id])?;
        Ok(())
    }

    pub fn find_by_id(conn: &Connection, id: i32) -> Result<Self> {
        let mut stmt = conn.prepare(
                "SELECT id, output_roll_no, final_meter, flag_reason, final_weight, core_weight, job_id, created_by, created_at, updated_at, from_batch, flag_count 
                 FROM rolls WHERE id = ?1"
            )?;
        stmt.query_row(params![id], |row| {
            Ok(Roll {
                id: row.get(0)?,
                output_roll_no: row.get(1)?,
                final_meter: row.get(2)?,
                flag_reason: row.get(3)?,
                final_weight: row.get(4)?,
                core_weight: row.get(5)?,
                job_id: row.get(6)?,
                created_by: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
                from_batch: row.get(10)?,
                flag_count: row.get(11)?,
            })
        })
    }

    pub fn all(conn: &Connection) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare(
                "SELECT id, output_roll_no, final_meter, flag_reason, final_weight, core_weight, job_id, created_by, created_at, updated_at, from_batch, flag_count 
                 FROM rolls ORDER BY created_at DESC"
        )?;
        let rolls = stmt
            .query_map([], |row| {
                Ok(Roll {
                    id: row.get(0)?,
                    output_roll_no: row.get(1)?,
                    final_meter: row.get(2)?,
                    flag_reason: row.get(3)?,
                    final_weight: row.get(4)?,
                    core_weight: row.get(5)?,
                    job_id: row.get(6)?,
                    created_by: row.get(7)?,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                    from_batch: row.get(10)?,
                    flag_count: row.get(11)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rolls)
    }

    pub fn filter(conn: &Connection, filter: &RollFilterPayload) -> Result<FilterResponse<Roll>> {
        let mut count_query =
            "SELECT COUNT(*) FROM rolls r JOIN jobs j ON r.job_id = j.id WHERE 1=1".to_string();
        let mut data_query =
            "SELECT r.id, r.output_roll_no, r.final_meter, r.flag_reason, r.final_weight, r.core_weight, r.job_id, r.created_by, r.created_at, r.updated_at, r.from_batch, r.flag_count 
             FROM rolls r JOIN jobs j ON r.job_id = j.id WHERE 1=1".to_string();
        let mut params_vec: Vec<&dyn rusqlite::ToSql> = vec![];
        // Hold boxed section_ids for params_vec lifetime
        let mut boxed_section_ids: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        let mut job_ids: Vec<i32> = vec![];
        let mut flag_reasons: Vec<String> = vec![];
        let mut created_bys: Vec<i32> = vec![];
        let mut output_roll_nos: Vec<String> = vec![];
        let mut start_dates: Vec<String> = vec![];
        let mut end_dates: Vec<String> = vec![];
        let mut shift_ids: Vec<i32> = vec![];
        let mut pages: Vec<i32> = vec![];
        let mut per_pages: Vec<i32> = vec![];
        let mut production_orders: Vec<String> = vec![];
        if let Some(val) = &filter.production_order {
            if !val.is_empty() {
                production_orders.push(val.clone());
                params_vec.push(production_orders.last().unwrap());
                count_query.push_str(" AND j.production_order = ?");
                data_query.push_str(" AND j.production_order = ?");
            }
        }
        if let Some(section_ids_vec) = &filter.section_ids {
            let valid_ids: Vec<i32> = section_ids_vec
                .iter()
                .filter_map(|val| val.parse::<i32>().ok())
                .collect();
            if !valid_ids.is_empty() {
                let placeholders = valid_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
                let clause = format!(" AND j.section_id IN ({})", placeholders);
                count_query.push_str(&clause);
                data_query.push_str(&clause);
                for id in valid_ids {
                    boxed_section_ids.push(Box::new(id));
                }
                for id in &boxed_section_ids {
                    params_vec.push(id.as_ref());
                }
            }
        }

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
                count_query.push_str(" AND (r.flag_reason IS NOT NULL AND r.flag_reason != '')");
                data_query.push_str(" AND (r.flag_reason IS NOT NULL AND r.flag_reason != '')");
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

        if let Some(val) = &filter.flag_reason {
            flag_reasons.push(val.to_string());
            params_vec.push(flag_reasons.last().unwrap());
            count_query.push_str(" AND r.flag_reason = ?");
            data_query.push_str(" AND r.flag_reason = ?");
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

        let total_count: i32 =
            conn.query_row(&count_query, params_vec.as_slice(), |row| row.get(0))?;

        data_query.push_str(" ORDER BY r.created_at DESC");

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

        let mut stmt = conn.prepare(&data_query)?;
        let rows = stmt.query_map(params_vec.as_slice(), |row| {
            Ok(Roll {
                id: row.get(0)?,
                output_roll_no: row.get(1)?,
                final_meter: row.get(2)?,
                flag_reason: row.get(3)?,
                final_weight: row.get(4)?,
                core_weight: row.get(5)?,
                job_id: row.get(6)?,
                created_by: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
                from_batch: row.get(10)?,
                flag_count: row.get(11)?,
            })
        })?;

        let data = rows.collect::<Result<Vec<_>, _>>()?;

        Ok(FilterResponse { total_count, data })
    }

    pub fn get_details(conn: &Connection, roll_id: i32) -> Result<RollDetails> {
        let mut stmt = conn.prepare(
            "SELECT r.output_roll_no, r.final_weight, r.final_meter, r.flag_reason, r.created_at,
                j.production_order, j.material_number, j.machine_id,
                po.description, r.flag_count
         FROM rolls r
         JOIN jobs j ON r.job_id = j.id
         LEFT JOIN process_order po ON j.production_order = po.process_order
         WHERE r.id = ?1",
        )?;

        stmt.query_row(params![roll_id], |row| {
            let output_roll_no: String = row.get(0)?;
            let final_weight: f64 = row.get(1)?;
            let final_meter: f64 = row.get(2)?;
            let flag_reason_raw: Option<String> = row.get(3)?;
            let created_at: String = row.get(4)?;
            let production_order: String = row.get(5)?;
            let material_number: Option<String> = row.get(6)?;
            let machine_id: Option<i32> = row.get(7)?;
            let process_order_description: Option<String> = row.get(8)?;
            let flag_count: i32 = row.get(9)?;

            let material_description = material_number.as_ref().and_then(|num| {
                conn.query_row(
                    "SELECT desc FROM materials_value_description WHERE value = ?",
                    params![num],
                    |row| row.get::<_, String>(0),
                )
                .ok()
            });

            let section = machine_id
                .and_then(|id| {
                    conn.query_row(
                        "SELECT name FROM machines WHERE id = ?",
                        params![id],
                        |row| row.get::<_, String>(0),
                    )
                    .ok()
                })
                .unwrap_or_default();

            // Parse flag_reason as IDs, fetch names, join with ','
            let flag_reason = if let Some(ref ids_str) = flag_reason_raw {
                let ids: Vec<&str> = ids_str
                    .split('|')
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .collect();
                if !ids.is_empty() {
                    let mut names = Vec::new();
                    for id in ids {
                        if let Ok(flag_name) = conn.query_row(
                            "SELECT name FROM flag_reasons WHERE id = ?1",
                            params![id],
                            |row| row.get::<_, String>(0),
                        ) {
                            names.push(flag_name);
                        }
                    }
                    if !names.is_empty() {
                        Some(names.join(", "))
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            Ok(RollDetails {
                material_number: material_number.unwrap_or_default(),
                material_description: material_description.unwrap_or_default(),
                process_order_description: process_order_description.unwrap_or_default(),
                production_order,
                output_roll_no,
                final_weight,
                final_meter,
                flag_reason,
                created_at,
                section,
                flag_count,
            })
        })
    }
}
