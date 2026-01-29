use std::io::{Error, ErrorKind};

use crate::{
    backend::models::FilterResponse,
    sap::{RollData, post_rolls},
};
use chrono::{Datelike, Local};
use rusqlite::{Connection, OptionalExtension, Result, params};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct OutputRoll {
    pub id: i32,
    pub output_batch: String,
    pub final_meter: f64,
    pub flag_reason: Option<String>,
    pub final_weight: f64,
    pub core_weight: Option<f64>,
    pub input_roll_id: i32,
    pub job_id: i32,
    pub from_input_batch: String,
    pub flag_count: i32,
    pub created_by: i32,
    pub operator_name: Option<String>,
    pub updated_by: Option<i32>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Deserialize)]
pub struct OutputRollCreatePayload {
    pub final_meter: f64,
    pub batch: String,
    pub flag_reason: Option<String>,
    pub core_weight: Option<f64>,
    pub shift_id: i32,
    pub job_id: i32,
    pub machine_id: i32,
    pub input_roll_id: i32,
    pub flag_count: i32,
}

#[derive(Deserialize)]
pub struct OutputRollPayload {
    pub id: i32,
    pub output_batch: Option<String>,
    pub final_meter: Option<f64>,
    pub flag_reason: Option<String>,
    pub final_weight: Option<f64>,
    pub core_weight: Option<f64>,
    pub input_roll_id: Option<i32>,
    pub from_input_batch: Option<String>,
    pub flag_count: Option<i32>,
    pub created_by: Option<i32>,
    pub updated_by: Option<i32>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Deserialize)]
pub struct OutputRollFilterPayload {
    pub job_id: Option<String>,
    pub shift_id: Option<String>,
    pub output_batch: Option<String>,
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
pub struct OutputRollDetails {
    pub material_number: String,
    pub material_description: String,
    pub process_order_description: String,
    pub production_order: String,
    pub output_batch: String,
    pub final_weight: f64,
    pub final_meter: f64,
    pub flag_reason: Option<String>,
    pub created_at: String,
    pub section: String,
    pub flag_count: i32,
    pub operator_name: Option<String>,
}

impl OutputRoll {
    pub fn build_from_input_batch(
        conn: &mut Connection,
        job_id: i64,
        input_roll_id: i64,
        current_batch: &str,
    ) -> Result<String> {
        let mut unconsumed: Vec<(i64, String)> = vec![];

        {
            let mut stmt = conn.prepare(
                r#"
            SELECT id, batch
            FROM input_rolls
            WHERE job_id = ?
              AND is_consumed = 0
            ORDER BY id ASC
            "#,
            )?;

            let rows = stmt.query_map(params![job_id], |r| {
                Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?))
            })?;

            for row in rows {
                unconsumed.push(row?);
            }
        }

        let current_is_unconsumed = unconsumed.iter().any(|(_, b)| b == current_batch);

        let prev_output_batch: Option<String> = conn
            .query_row(
                r#"
            SELECT ir.batch
            FROM output_rolls o
            JOIN input_rolls ir ON o.input_roll_id = ir.id
            WHERE ir.job_id = ?
            ORDER BY COALESCE(o.created_at, o.updated_at) DESC, o.id DESC
            LIMIT 1
            "#,
                params![job_id],
                |r| r.get(0),
            )
            .optional()?;

        let mut include_ids: Vec<i64> = vec![];
        let mut batches: Vec<String> = vec![];

        if current_is_unconsumed {
            if let Some(p) = prev_output_batch {
                batches.push(p);
            }
            for (id, b) in &unconsumed {
                include_ids.push(*id);
                batches.push(b.clone());
            }
            if !include_ids.contains(&input_roll_id) {
                include_ids.push(input_roll_id);
            }
        } else {
            batches.push(current_batch.to_string());
        }

        include_ids.sort_unstable();
        include_ids.dedup();

        let from_input = batches.join(", ");

        if current_is_unconsumed {
            let now = chrono::Utc::now().naive_utc().to_string();
            let tx = conn.transaction()?;
            for id in &include_ids {
                tx.execute(
                "UPDATE input_rolls SET is_consumed = 1, consumed_at = ? WHERE id = ? AND job_id = ?",
                params![now, id, job_id],
            )?;
            }
            tx.commit()?;
        }

        Ok(from_input)
    }

    pub fn create(
        conn: &mut Connection,
        data: &OutputRollCreatePayload,
        user_id: i32,
    ) -> Result<Self> {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let machine_id = data.machine_id;
        let shift_id = data.shift_id;
        let current_batch = &data.batch;
        let job_id = data.job_id;
        let input_roll_id = data.input_roll_id;

        let machine = conn.query_row(
            "SELECT label FROM machines WHERE id = ?1",
            params![machine_id],
            |row| row.get::<_, String>(0),
        )?;

        let current_date = Local::now();
        let year = current_date.format("%y").to_string();
        let day_of_year = current_date.date_naive().ordinal() as i32;
        let shift_number = (day_of_year - 1) * 2 + shift_id;
        let today_start = current_date.format("%Y-%m-%d").to_string();

        let from_batch = OutputRoll::build_from_input_batch(
            conn,
            job_id as i64,
            input_roll_id as i64,
            current_batch,
        )?;
        let roll_count_for_job: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM output_rolls o JOIN input_rolls ir ON o.input_roll_id = ir.id WHERE ir.job_id = ?1 AND date(o.created_at) = date(?2)",
                params![job_id, today_start],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let roll_number = roll_count_for_job + 1;
        let output_batch = format!("{}{:03}{}{:03}", year, shift_number, machine, roll_number);

        conn.execute(
            "INSERT INTO output_rolls (output_batch, final_meter, flag_reason, final_weight, core_weight, input_roll_id, created_by, created_at, updated_at, from_input_batch, flag_count) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![output_batch, data.final_meter, data.flag_reason, 0.0, data.core_weight, data.input_roll_id, user_id, now, now, from_batch, data.flag_count],
        )?;

        let id = conn.last_insert_rowid() as i32;
        Ok(OutputRoll {
            id,
            output_batch,
            final_meter: data.final_meter,
            flag_reason: data.flag_reason.clone(),
            final_weight: 0.0,
            core_weight: data.core_weight,
            input_roll_id: data.input_roll_id,
            job_id,
            created_by: user_id,
            operator_name: None,
            updated_by: None,
            created_at: now.clone(),
            updated_at: now.clone(),
            from_input_batch: from_batch,
            flag_count: data.flag_count,
        })
    }

    pub async fn update(&mut self, conn: &Connection, data: &OutputRollPayload) -> Result<()> {
        if let Some(final_weight) = data.final_weight {
            // Fetch job info via input_rolls (using input_roll_id)
            let (job_production_order,): (String,) = conn.query_row(
                "SELECT j.production_order FROM input_rolls ir JOIN jobs j ON ir.job_id = j.id WHERE ir.id = ?1",
                params![self.input_roll_id],
                |row| Ok((row.get(0)?,)),
            )?;

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
                batch: self.output_batch.to_string(),
                production_order: job_production_order,
            };

            if let Err(e) = post_rolls(roll_data).await {
                return Err(rusqlite::Error::ToSqlConversionFailure(Box::new(
                    Error::new(ErrorKind::Other, e),
                )));
            }

            conn.execute(
                "UPDATE output_rolls SET final_meter = ?1, final_weight = ?2 WHERE id = ?3",
                params![new_alternate_quantity, net_weight, self.id],
            )?;
            self.final_meter = new_alternate_quantity;
            self.final_weight = net_weight;
        }

        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        if let Some(output_batch) = &data.output_batch {
            conn.execute(
                "UPDATE output_rolls SET output_batch = ?1 WHERE id = ?2",
                params![output_batch, self.id],
            )?;
            self.output_batch = output_batch.clone();
        }
        if let Some(final_meter) = data.final_meter {
            conn.execute(
                "UPDATE output_rolls SET final_meter = ?1 WHERE id = ?2",
                params![final_meter, self.id],
            )?;
            self.final_meter = final_meter;
        }
        if let Some(flag_reason) = &data.flag_reason {
            conn.execute(
                "UPDATE output_rolls SET flag_reason = ?1 WHERE id = ?2",
                params![flag_reason, self.id],
            )?;
            self.flag_reason = Some(flag_reason.to_string());
        }
        if let Some(input_roll_id) = data.input_roll_id {
            conn.execute(
                "UPDATE output_rolls SET input_roll_id = ?1 WHERE id = ?2",
                params![input_roll_id, self.id],
            )?;
            self.input_roll_id = input_roll_id;
        }
        if let Some(from_input_batch) = &data.from_input_batch {
            conn.execute(
                "UPDATE output_rolls SET from_input_batch = ?1 WHERE id = ?2",
                params![from_input_batch, self.id],
            )?;
            self.from_input_batch = from_input_batch.clone();
        }
        if let Some(updated_by) = data.updated_by {
            conn.execute(
                "UPDATE output_rolls SET updated_by = ?1 WHERE id = ?2",
                params![updated_by, self.id],
            )?;
            self.updated_by = Some(updated_by);
        }
        conn.execute(
            "UPDATE output_rolls SET updated_at = ?1 WHERE id = ?2",
            params![now, self.id],
        )?;
        self.updated_at = now;
        Ok(())
    }

    pub fn delete(&self, conn: &Connection) -> Result<()> {
        conn.execute("DELETE FROM output_rolls WHERE id = ?1", params![self.id])?;
        Ok(())
    }

    pub fn find_by_id(conn: &Connection, id: i32) -> Result<Self> {
        let mut stmt = conn.prepare(
                "SELECT o.id, o.output_batch, o.final_meter, o.flag_reason, o.final_weight, o.core_weight, o.input_roll_id, j.id as job_id, o.created_by, u.full_name, o.updated_by, o.created_at, o.updated_at, o.from_input_batch, o.flag_count \
                 FROM output_rolls o \
                 JOIN input_rolls ir ON o.input_roll_id = ir.id \
                 JOIN jobs j ON ir.job_id = j.id \
                 LEFT JOIN users u ON o.created_by = u.id \
                 WHERE o.id = ?1"
            )?;
        stmt.query_row(params![id], |row| {
            Ok(OutputRoll {
                id: row.get(0)?,
                output_batch: row.get(1)?,
                final_meter: row.get(2)?,
                flag_reason: row.get(3)?,
                final_weight: row.get(4)?,
                core_weight: row.get(5)?,
                input_roll_id: row.get(6)?,
                job_id: row.get(7)?,
                created_by: row.get(8)?,
                operator_name: row.get(9)?,
                updated_by: row.get(10)?,
                created_at: row.get(11)?,
                updated_at: row.get(12)?,
                from_input_batch: row.get(13)?,
                flag_count: row.get(14)?,
            })
        })
    }

    pub fn all(conn: &Connection) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare(
                "SELECT o.id, o.output_batch, o.final_meter, o.flag_reason, o.final_weight, o.core_weight, o.input_roll_id, j.id as job_id, o.created_by, u.full_name, o.updated_by, o.created_at, o.updated_at, o.from_input_batch, o.flag_count \
                 FROM output_rolls o \
                 JOIN input_rolls ir ON o.input_roll_id = ir.id \
                 JOIN jobs j ON ir.job_id = j.id \
                 LEFT JOIN users u ON o.created_by = u.id \
                 ORDER BY o.created_at DESC"
        )?;
        let rolls = stmt
            .query_map([], |row| {
                Ok(OutputRoll {
                    id: row.get(0)?,
                    output_batch: row.get(1)?,
                    final_meter: row.get(2)?,
                    flag_reason: row.get(3)?,
                    final_weight: row.get(4)?,
                    core_weight: row.get(5)?,
                    input_roll_id: row.get(6)?,
                    job_id: row.get(7)?,
                    created_by: row.get(8)?,
                    operator_name: row.get(9)?,
                    updated_by: row.get(10)?,
                    created_at: row.get(11)?,
                    updated_at: row.get(12)?,
                    from_input_batch: row.get(13)?,
                    flag_count: row.get(14)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rolls)
    }

    pub fn filter(
        conn: &Connection,
        filter: &OutputRollFilterPayload,
    ) -> Result<FilterResponse<OutputRoll>> {
        let mut count_query =
            "SELECT COUNT(*) FROM output_rolls r JOIN input_rolls ir ON r.input_roll_id = ir.id JOIN jobs j ON ir.job_id = j.id WHERE 1=1"
                .to_string();
        let mut data_query =
            "SELECT r.id, r.output_batch, r.final_meter, r.flag_reason, r.final_weight, r.core_weight, r.input_roll_id, j.id as job_id, r.created_by, u.full_name, r.updated_by, r.created_at, r.updated_at, r.from_input_batch, r.flag_count \
             FROM output_rolls r \
             JOIN input_rolls ir ON r.input_roll_id = ir.id \
             JOIN jobs j ON ir.job_id = j.id \
             LEFT JOIN users u ON r.created_by = u.id \
             WHERE 1=1".to_string();
        let mut params_vec: Vec<&dyn rusqlite::ToSql> = vec![];
        // Hold boxed section_ids for params_vec lifetime
        let mut boxed_section_ids: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        let mut job_ids: Vec<i32> = vec![];
        let mut flag_reasons: Vec<String> = vec![];
        let mut created_bys: Vec<i32> = vec![];
        let mut output_batchs: Vec<String> = vec![];
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
                count_query.push_str(" AND ir.job_id = ?");
                data_query.push_str(" AND ir.job_id = ?");
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

        if let Some(val) = &filter.output_batch {
            if !val.is_empty() {
                output_batchs.push(format!("%{}%", val));
                params_vec.push(output_batchs.last().unwrap());
                count_query.push_str(" AND r.output_batch LIKE ?");
                data_query.push_str(" AND r.output_batch LIKE ?");
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
            Ok(OutputRoll {
                id: row.get(0)?,
                output_batch: row.get(1)?,
                final_meter: row.get(2)?,
                flag_reason: row.get(3)?,
                final_weight: row.get(4)?,
                core_weight: row.get(5)?,
                input_roll_id: row.get(6)?,
                job_id: row.get(7)?,
                created_by: row.get(8)?,
                operator_name: row.get(9)?,
                updated_by: row.get(10)?,
                created_at: row.get(11)?,
                updated_at: row.get(12)?,
                from_input_batch: row.get(13)?,
                flag_count: row.get(14)?,
            })
        })?;

        let data = rows.collect::<Result<Vec<_>, _>>()?;

        Ok(FilterResponse { total_count, data })
    }

    pub fn get_details(conn: &Connection, roll_id: i32) -> Result<OutputRollDetails> {
        let mut stmt = conn.prepare(
            "SELECT r.output_batch, r.final_weight, r.final_meter, r.flag_reason, r.created_at,
                j.production_order, ir.material_number, j.machine_id,
                po.description, r.flag_count, u.full_name
         FROM output_rolls r
         JOIN input_rolls ir ON r.input_roll_id = ir.id
         JOIN jobs j ON ir.job_id = j.id
         LEFT JOIN process_order po ON j.production_order = po.process_order
         LEFT JOIN users u ON r.created_by = u.id
         WHERE r.id = ?1",
        )?;

        stmt.query_row(params![roll_id], |row| {
            let output_batch: String = row.get(0)?;
            let final_weight: f64 = row.get(1)?;
            let final_meter: f64 = row.get(2)?;
            let flag_reason_raw: Option<String> = row.get(3)?;
            let created_at: String = row.get(4)?;
            let production_order: String = row.get(5)?;
            let material_number: Option<String> = row.get(6)?;
            let machine_id: Option<i32> = row.get(7)?;
            let process_order_description: Option<String> = row.get(8)?;
            let flag_count: i32 = row.get(9)?;
            let operator_name: Option<String> = row.get(10)?;

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

            // Parse flag_reason as IDs (optionally with ":message"), fetch names, join with ','
            // Fallback to raw tokens when IDs don't resolve so PDF never shows null.
            let flag_reason = if let Some(ref ids_str) = flag_reason_raw {
                let parts: Vec<&str> = ids_str
                    .split('|')
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .collect();
                if parts.is_empty() {
                    None
                } else {
                    let mut names = Vec::new();
                    for part in parts {
                        let (id_part, msg_part) = match part.split_once(':') {
                            Some((id, msg)) => (id.trim(), Some(msg.trim())),
                            None => (part, None),
                        };

                        let resolved_name = id_part
                            .parse::<i32>()
                            .ok()
                            .and_then(|id| {
                                conn.query_row(
                                    "SELECT name FROM flag_reasons WHERE id = ?1",
                                    params![id],
                                    |row| row.get::<_, String>(0),
                                )
                                .ok()
                            });

                        let display = match (resolved_name, msg_part) {
                            (Some(name), Some(msg)) if !msg.is_empty() => format!("{}: {}", name, msg),
                            (Some(name), _) => name,
                            (None, _) => part.to_string(),
                        };

                        if !display.trim().is_empty() {
                            names.push(display);
                        }
                    }

                    if names.is_empty() {
                        None
                    } else {
                        Some(names.join(", "))
                    }
                }
            } else {
                None
            };

            Ok(OutputRollDetails {
                material_number: material_number.unwrap_or_default(),
                material_description: material_description.unwrap_or_default(),
                process_order_description: process_order_description.unwrap_or_default(),
                production_order,
                output_batch,
                final_weight,
                final_meter,
                flag_reason,
                created_at,
                section,
                flag_count,
                operator_name,
            })
        })
    }
}
