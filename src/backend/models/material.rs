use rusqlite::{params, Connection, Result};
use serde::{Serialize, Deserialize};
use crate::backend::models::FilterResponse;
use std::collections::HashMap;

#[derive(Debug, Serialize)]
pub struct Material {
    pub id: i32,
    pub code: String,
    pub key: String,
    pub value: String,
    pub created_at: String,
    pub material_details: HashMap<String, String>,
}

#[derive(Debug, Serialize)]
pub struct MaterialStats {
    pub total_materials: i32,
    pub total_without_descriptions: i32,
}

#[derive(Deserialize)]
pub struct MaterialCreatePayload {
    pub code: String,
}

#[derive(Deserialize)]
pub struct MaterialPayload {
    pub id: i32,
    pub code: Option<String>,
}

#[derive(Deserialize)]
pub struct MaterialFilterPayload {
    pub code: Option<String>,
    pub key: Option<String>,
    pub has_descriptions: Option<String>,
    pub per_page: Option<String>,
    pub page: Option<String>,
}

impl Material {
    pub fn create(conn: &Connection, data: &MaterialCreatePayload) -> Result<Self> {
        conn.execute(
            "INSERT INTO materials (code, key, value, created_at) VALUES (?1, 'Loading...', 'Loading...', datetime('now'))",
            params![data.code],
        )?;
        let id = conn.last_insert_rowid() as i32;
        
        Self::find_by_id(conn, id)
    }

    pub fn update(&mut self, conn: &Connection, data: &MaterialPayload) -> Result<()> {
        if let Some(code) = &data.code {
            conn.execute("UPDATE materials SET code = ?1 WHERE id = ?2", params![code, self.id])?;
            self.code = code.clone();
        }
        Ok(())
    }

    pub fn delete(&self, conn: &Connection) -> Result<()> {
        conn.execute("DELETE FROM materials WHERE id = ?1", params![self.id])?;
        Ok(())
    }

    pub fn find_by_id(conn: &Connection, id: i32) -> Result<Self> {
        let mut stmt = conn.prepare(
            "SELECT id, code, key, value, created_at FROM materials WHERE id = ?1"
        )?;
        
        stmt.query_row(params![id], |row| {
            let key: String = row.get(2)?;
            let value: String = row.get(3)?;
            let material_details = Self::get_material_details(conn, &key, &value)?;
            
            Ok(Material {
                id: row.get(0)?,
                code: row.get(1)?,
                key,
                value,
                created_at: row.get(4)?,
                material_details,
            })
        })
    }

    pub fn find_by_code(conn: &Connection, code: &str) -> Result<Self> {
        let mut stmt = conn.prepare(
            "SELECT id, code, key, value, created_at FROM materials WHERE code = ?1"
        )?;
        
        stmt.query_row(params![code], |row| {
            let key: String = row.get(2)?;
            let value: String = row.get(3)?;
            let material_details = Self::get_material_details(conn, &key, &value)?;
            
            Ok(Material {
                id: row.get(0)?,
                code: row.get(1)?,
                key,
                value,
                created_at: row.get(4)?,
                material_details,
            })
        })
    }

    pub fn all(conn: &Connection) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare(
            "SELECT id, code, key, value, created_at FROM materials ORDER BY code"
        )?;
        let materials = stmt.query_map([], |row| {
            let key: String = row.get(2)?;
            let value: String = row.get(3)?;
            let material_details = Self::get_material_details(conn, &key, &value)?;
            
            Ok(Material {
                id: row.get(0)?,
                code: row.get(1)?,
                key,
                value,
                created_at: row.get(4)?,
                material_details,
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        Ok(materials)
    }

    pub fn get_stats(conn: &Connection) -> Result<MaterialStats> {
        let total_materials: i32 = conn.query_row("SELECT COUNT(*) FROM materials", [], |row| row.get(0))?;
        let total_without_descriptions: i32 = conn.query_row(
            "SELECT COUNT(DISTINCT mvd.id) FROM materials_value_description mvd WHERE mvd.desc IS NULL OR mvd.desc = ''",
            [],
            |row| row.get(0)
        )?;

        Ok(MaterialStats {
            total_materials,
            total_without_descriptions,
        })
    }

    pub fn filter(conn: &Connection, filter: &MaterialFilterPayload) -> Result<FilterResponse<Self>> {
        let mut count_query = "SELECT COUNT(*) FROM materials m WHERE 1=1".to_string();
        let mut data_query = "SELECT m.id, m.code, m.key, m.value, m.created_at FROM materials m WHERE 1=1".to_string();
        let mut params_vec: Vec<&dyn rusqlite::ToSql> = vec![];

        let mut codes: Vec<String> = vec![];
        let mut keys: Vec<String> = vec![];
        let mut pages: Vec<i32> = vec![];
        let mut per_pages: Vec<i32> = vec![];

        if let Some(val) = &filter.code {
            if !val.is_empty() {
                codes.push(format!("%{}%", val));
                params_vec.push(codes.last().unwrap());
                count_query.push_str(" AND m.code LIKE ?");
                data_query.push_str(" AND m.code LIKE ?");
            }
        }

        if let Some(val) = &filter.key {
            if !val.is_empty() {
                keys.push(format!("%{}%", val));
                params_vec.push(keys.last().unwrap());
                count_query.push_str(" AND m.key LIKE ?");
                data_query.push_str(" AND m.key LIKE ?");
            }
        }

        if let Some(val) = &filter.has_descriptions {
            if val == "true" {
                count_query.push_str(" AND NOT (m.key = 'Loading...' OR m.value = 'Loading...')");
                data_query.push_str(" AND NOT (m.key = 'Loading...' OR m.value = 'Loading...')");
            } else if val == "false" {
                count_query.push_str(" AND (m.key = 'Loading...' OR m.value = 'Loading...')");
                data_query.push_str(" AND (m.key = 'Loading...' OR m.value = 'Loading...')");
            }
        }

        let total_count: i32 = conn.query_row(&count_query, params_vec.as_slice(), |row| row.get(0))?;

        data_query.push_str(" ORDER BY m.code");

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
            let key: String = row.get(2)?;
            let value: String = row.get(3)?;
            let material_details = Self::get_material_details(conn, &key, &value)?;
            
            Ok(Material {
                id: row.get(0)?,
                code: row.get(1)?,
                key,
                value,
                created_at: row.get(4)?,
                material_details,
            })
        })?;

        let data = rows.collect::<Result<Vec<_>, _>>()?;

        Ok(FilterResponse {
            total_count,
            data,
        })
    }

    fn get_material_details(conn: &Connection, key_str: &str, value_str: &str) -> Result<HashMap<String, String>> {
        let mut details = HashMap::new();
        let keys: Vec<&str> = key_str.split(',').collect();
        let value_ids: Vec<&str> = value_str.split(',').collect();
        
        for (key, id_str) in keys.into_iter().zip(value_ids.into_iter()) {
            if !key.is_empty() && !id_str.is_empty() {
                if let Ok(id) = id_str.parse::<i64>() {
                    if let Ok(desc) = conn.query_row(
                        "SELECT desc FROM materials_value_description WHERE id = ?",
                        params![id],
                        |row| row.get::<_, String>(0)
                    ) {
                        details.entry(key.to_string())
                            .and_modify(|v: &mut String| v.push_str(&format!(",{}", desc)))
                            .or_insert(desc);
                    }
                }
            }
        }
        Ok(details)
    }
}