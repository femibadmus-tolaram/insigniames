use crate::backend::models::FilterResponse;
use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize)]
pub struct ProcessOrder {
    pub id: i32,
    pub process_order: String,
    pub posting_date: String,
    pub shift: String,
    pub description: String,
    pub line: String,
    pub po_code_id: i32,
    pub material_id: i32,
    pub material_numbers: HashMap<String, String>,
    pub material_details: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct ProcessOrderFilterPayload {
    pub process_order: Option<String>,
    pub line: Option<String>,
    pub shift: Option<String>,
    pub from_date: Option<String>,
    pub to_date: Option<String>,
    pub per_page: Option<String>,
    pub page: Option<String>,
    pub section_ids: String,
    pub posting_date: Option<String>,
}

impl ProcessOrder {
    pub fn all(conn: &Connection) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare(
            "SELECT po.rowid as id, po.*, m.key as material_key, m.value as material_value 
             FROM process_order po
             LEFT JOIN materials m ON po.material_id = m.id
             ORDER BY po.posting_date DESC, po.process_order",
        )?;
        let orders = stmt
            .query_map([], |row| {
                let material_key: String = row.get(8)?;
                let material_value: String = row.get(9)?;
                let material_details =
                    Self::group_material_details(&conn, &material_key, &material_value);
                let material_numbers =
                    Self::group_material_numbers(&conn, &material_key, &material_value);

                Ok(ProcessOrder {
                    id: row.get(0)?,
                    process_order: row.get(1)?,
                    posting_date: row.get(2)?,
                    shift: row.get(3)?,
                    description: row.get(4)?,
                    line: row.get(5)?,
                    po_code_id: row.get(6)?,
                    material_id: row.get(7)?,
                    material_details,
                    material_numbers,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(orders)
    }

    pub fn filter(
        conn: &Connection,
        filter: &ProcessOrderFilterPayload,
    ) -> Result<FilterResponse<Self>> {
        let section_ids: Vec<i32> = filter
            .section_ids
            .split(',')
            .filter_map(|s| s.parse().ok())
            .collect();

        if section_ids.is_empty() {
            return Ok(FilterResponse {
                total_count: 0,
                data: vec![],
            });
        }

        let section_ids_str = section_ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<String>>()
            .join(",");

        let mut count_query = format!(
            "SELECT COUNT(DISTINCT po.rowid) FROM process_order po WHERE EXISTS (SELECT 1 FROM po_code_sections pcs WHERE pcs.po_code_id = po.po_code_id AND pcs.section_id IN ({}))",
            section_ids_str
        );

        let mut data_query = format!(
            "SELECT po.rowid as id, po.*, m.key as material_key, m.value as material_value FROM process_order po LEFT JOIN materials m ON po.material_id = m.id WHERE EXISTS (SELECT 1 FROM po_code_sections pcs WHERE pcs.po_code_id = po.po_code_id AND pcs.section_id IN ({}))",
            section_ids_str
        );

        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![];

        if let Some(process_order) = &filter.process_order {
            if !process_order.is_empty() {
                count_query.push_str(" AND po.process_order LIKE ?");
                data_query.push_str(" AND po.process_order LIKE ?");
                params_vec.push(Box::new(format!("%{}%", process_order)));
            }
        }

        if let Some(posting_date) = &filter.posting_date {
            if !posting_date.is_empty() {
                count_query.push_str(" AND po.posting_date = ?");
                data_query.push_str(" AND po.posting_date = ?");
                params_vec.push(Box::new(posting_date.clone()));
            }
        }

        if let Some(line) = &filter.line {
            if !line.is_empty() {
                count_query.push_str(" AND po.line = ?");
                data_query.push_str(" AND po.line = ?");
                params_vec.push(Box::new(line.clone()));
            }
        }

        if let Some(shift) = &filter.shift {
            if !shift.is_empty() {
                count_query.push_str(" AND po.shift = ?");
                data_query.push_str(" AND po.shift = ?");
                params_vec.push(Box::new(shift.clone()));
            }
        }

        if let Some(from_date) = &filter.from_date {
            if !from_date.is_empty() {
                count_query.push_str(" AND po.posting_date >= ?");
                data_query.push_str(" AND po.posting_date >= ?");
                params_vec.push(Box::new(from_date.clone()));
            }
        }

        if let Some(to_date) = &filter.to_date {
            if !to_date.is_empty() {
                count_query.push_str(" AND po.posting_date <= ?");
                data_query.push_str(" AND po.posting_date <= ?");
                params_vec.push(Box::new(to_date.clone()));
            }
        }

        let count_params: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| &**p).collect();
        let total_count: i32 =
            conn.query_row(&count_query, count_params.as_slice(), |row| row.get(0))?;

        data_query.push_str(" GROUP BY po.rowid ORDER BY po.posting_date DESC, po.process_order");

        if let (Some(page), Some(per_page)) = (&filter.page, &filter.per_page) {
            if let (Ok(page_val), Ok(per_page_val)) = (page.parse::<i32>(), per_page.parse::<i32>())
            {
                if per_page_val > 0 {
                    let offset = (page_val - 1) * per_page_val;
                    data_query.push_str(" LIMIT ? OFFSET ?");
                    params_vec.push(Box::new(per_page_val));
                    params_vec.push(Box::new(offset));
                }
            }
        }

        let data_params: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| &**p).collect();
        let mut stmt = conn.prepare(&data_query)?;
        let rows = stmt.query_map(data_params.as_slice(), |row| {
            let material_key: String = row.get(8)?;
            let material_value: String = row.get(9)?;
            let material_details =
                Self::group_material_details(&conn, &material_key, &material_value);
            let material_numbers =
                Self::group_material_numbers(&conn, &material_key, &material_value);

            Ok(ProcessOrder {
                id: row.get(0)?,
                process_order: row.get(1)?,
                posting_date: row.get(2)?,
                shift: row.get(3)?,
                description: row.get(4)?,
                line: row.get(5)?,
                po_code_id: row.get(6)?,
                material_id: row.get(7)?,
                material_details,
                material_numbers,
            })
        })?;

        let data = rows.collect::<Result<Vec<_>, _>>()?;

        Ok(FilterResponse { total_count, data })
    }

    fn group_material_details(
        conn: &Connection,
        key_str: &str,
        value_str: &str,
    ) -> HashMap<String, String> {
        let mut details = HashMap::new();
        let keys: Vec<&str> = key_str.split(',').collect();
        let value_ids: Vec<&str> = value_str.split(',').collect();

        for (key, id_str) in keys.into_iter().zip(value_ids.into_iter()) {
            if !key.is_empty() && !id_str.is_empty() {
                if let Ok(id) = id_str.parse::<i64>() {
                    if let Ok(desc) = conn.query_row(
                        "SELECT desc FROM materials_value_description WHERE id = ?",
                        params![id],
                        |row| row.get::<_, String>(0),
                    ) {
                        details
                            .entry(key.to_string())
                            .and_modify(|v: &mut String| v.push_str(&format!(",{}", desc)))
                            .or_insert(desc);
                    }
                }
            }
        }
        details
    }

    fn group_material_numbers(
        conn: &Connection,
        key_str: &str,
        value_str: &str,
    ) -> HashMap<String, String> {
        let mut numbers = HashMap::new();
        let keys: Vec<&str> = key_str.split(',').collect();
        let value_ids: Vec<&str> = value_str.split(',').collect();

        for (key, id_str) in keys.into_iter().zip(value_ids.into_iter()) {
            if !key.is_empty() && !id_str.is_empty() {
                if let Ok(id) = id_str.parse::<i64>() {
                    if let Ok(number) = conn.query_row(
                        "SELECT value FROM materials_value_description WHERE id = ?",
                        params![id],
                        |row| row.get::<_, String>(0),
                    ) {
                        numbers
                            .entry(key.to_string())
                            .and_modify(|v: &mut String| v.push_str(&format!(",{}", number)))
                            .or_insert(number);
                    }
                }
            }
        }
        numbers
    }
}
