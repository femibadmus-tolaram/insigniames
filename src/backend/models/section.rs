use rusqlite::{params, Connection, Result};
use serde::{Serialize, Deserialize};
use crate::backend::models::FilterResponse;

#[derive(Debug, Serialize)]
pub struct Section {
    pub id: i32,
    pub name: String,
    pub machine_count: i32,
    pub user_count: i32,
    pub order_type_ids: Vec<i32>,
}

#[derive(Debug, Serialize)]
pub struct SectionStats {
    pub total_sections: i32,
    pub total_machines: i32,
    pub total_users: i32,
    pub total_order_types: i32,
}

#[derive(Deserialize)]
pub struct SectionCreatePayload {
    pub name: String,
}

#[derive(Deserialize)]
pub struct SectionPayload {
    pub id: i32,
    pub name: Option<String>,
    pub order_type_ids: Option<Vec<i32>>,
}

#[derive(Deserialize)]
pub struct SectionFilterPayload {
    pub name: Option<String>,
    pub has_machines: Option<String>,
    pub has_users: Option<String>,
    pub has_order_types: Option<String>,
    pub per_page: Option<String>,
    pub page: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SectionWithDetails {
    pub id: i32,
    pub name: String,
    pub machines: Vec<MachineInfo>,
    pub order_types: Vec<OrderTypeInfo>,
    pub users: Vec<UserInfo>,
}

#[derive(Debug, Serialize)]
pub struct MachineInfo {
    pub id: i32,
    pub name: String,
    pub label: String,
    pub job_count: i32,
}

#[derive(Debug, Serialize)]
pub struct OrderTypeInfo {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: i32,
    pub full_name: String,
    pub staffid: String,
}

impl Section {
    pub fn create(conn: &Connection, data: &SectionCreatePayload) -> Result<Self> {
        conn.execute(
            "INSERT INTO sections (name) VALUES (?1)",
            params![data.name],
        )?;
        let id = conn.last_insert_rowid() as i32;
        
        let mut stmt = conn.prepare(
            "SELECT s.*,
             COUNT(DISTINCT m.id) as machine_count,
             COUNT(DISTINCT us.user_id) as user_count
             FROM sections s
             LEFT JOIN machines m ON s.id = m.section_id
             LEFT JOIN user_sections us ON s.id = us.section_id
             WHERE s.id = ?1
             GROUP BY s.id"
        )?;
        
        let order_type_ids = Self::get_order_type_ids(conn, id)?;
        
        stmt.query_row(params![id], |row| Ok(Section {
            id: row.get(0)?,
            name: row.get(1)?,
            machine_count: row.get(2)?,
            user_count: row.get(3)?,
            order_type_ids,
        }))
    }

    pub fn update(&mut self, conn: &Connection, data: &SectionPayload) -> Result<()> {
        if let Some(name) = &data.name {
            conn.execute("UPDATE sections SET name = ?1 WHERE id = ?2", params![name, self.id])?;
            self.name = name.clone();
        }
        
        if let Some(order_type_ids) = &data.order_type_ids {
            conn.execute("DELETE FROM section_order_types WHERE section_id = ?1", params![self.id])?;
            
            for &order_type_id in order_type_ids {
                conn.execute(
                    "INSERT INTO section_order_types (section_id, order_type_id) VALUES (?1, ?2)",
                    params![self.id, order_type_id],
                )?;
            }
            self.order_type_ids = order_type_ids.clone();
        }
        
        Ok(())
    }

    pub fn delete(&self, conn: &Connection) -> Result<()> {
        conn.execute("DELETE FROM section_order_types WHERE section_id = ?1", params![self.id])?;
        conn.execute("DELETE FROM user_sections WHERE section_id = ?1", params![self.id])?;
        conn.execute("UPDATE machines SET section_id = NULL WHERE section_id = ?1", params![self.id])?;
        conn.execute("DELETE FROM sections WHERE id = ?1", params![self.id])?;
        Ok(())
    }

    pub fn has_machines(conn: &Connection, section_id: i32) -> Result<bool> {
        let count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM machines WHERE section_id = ?1",
            params![section_id],
            |row| row.get(0)
        )?;
        Ok(count > 0)
    }

    fn get_order_type_ids(conn: &Connection, section_id: i32) -> Result<Vec<i32>> {
        let mut stmt = conn.prepare(
            "SELECT order_type_id FROM section_order_types WHERE section_id = ?1 ORDER BY order_type_id"
        )?;
        let rows = stmt.query_map(params![section_id], |row| row.get(0))?;
        rows.collect()
    }

    pub fn find_by_id(conn: &Connection, id: i32) -> Result<Self> {
        let mut stmt = conn.prepare(
            "SELECT s.*,
             COUNT(DISTINCT m.id) as machine_count,
             COUNT(DISTINCT us.user_id) as user_count
             FROM sections s
             LEFT JOIN machines m ON s.id = m.section_id
             LEFT JOIN user_sections us ON s.id = us.section_id
             WHERE s.id = ?1
             GROUP BY s.id"
        )?;
        
        let order_type_ids = Self::get_order_type_ids(conn, id)?;
        
        stmt.query_row(params![id], |row| Ok(Section {
            id: row.get(0)?,
            name: row.get(1)?,
            machine_count: row.get(2)?,
            user_count: row.get(3)?,
            order_type_ids,
        }))
    }

    pub fn all(conn: &Connection) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare(
            "SELECT s.*,
             COUNT(DISTINCT m.id) as machine_count,
             COUNT(DISTINCT us.user_id) as user_count
             FROM sections s
             LEFT JOIN machines m ON s.id = m.section_id
             LEFT JOIN user_sections us ON s.id = us.section_id
             GROUP BY s.id
             ORDER BY s.name"
        )?;
        let sections = stmt.query_map([], |row| {
            let id: i32 = row.get(0)?;
            let order_type_ids = Self::get_order_type_ids(conn, id)?;
            Ok(Section {
                id,
                name: row.get(1)?,
                machine_count: row.get(2)?,
                user_count: row.get(3)?,
                order_type_ids,
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        Ok(sections)
    }

    pub fn get_stats(conn: &Connection) -> Result<SectionStats> {
        let total_sections: i32 = conn.query_row("SELECT COUNT(*) FROM sections", [], |row| row.get(0))?;
        let total_machines: i32 = conn.query_row("SELECT COUNT(*) FROM machines", [], |row| row.get(0))?;
        let total_users: i32 = conn.query_row("SELECT COUNT(DISTINCT user_id) FROM user_sections", [], |row| row.get(0))?;
        let total_order_types: i32 = conn.query_row("SELECT COUNT(DISTINCT order_type_id) FROM section_order_types", [], |row| row.get(0))?;

        Ok(SectionStats {
            total_sections,
            total_machines,
            total_users,
            total_order_types,
        })
    }

    pub fn filter(conn: &Connection, filter: &SectionFilterPayload) -> Result<FilterResponse<Self>> {
        let mut count_query = "SELECT COUNT(*) FROM sections s WHERE 1=1".to_string();
        let mut data_query = "SELECT s.*,
                             COUNT(DISTINCT m.id) as machine_count,
                             COUNT(DISTINCT us.user_id) as user_count
                             FROM sections s
                             LEFT JOIN machines m ON s.id = m.section_id
                             LEFT JOIN user_sections us ON s.id = us.section_id
                             WHERE 1=1".to_string();
        let mut params_vec: Vec<&dyn rusqlite::ToSql> = vec![];

        let mut names: Vec<String> = vec![];
        let mut pages: Vec<i32> = vec![];
        let mut per_pages: Vec<i32> = vec![];

        if let Some(val) = &filter.name {
            if !val.is_empty() {
                names.push(format!("%{}%", val));
                params_vec.push(names.last().unwrap());
                count_query.push_str(" AND s.name LIKE ?");
                data_query.push_str(" AND s.name LIKE ?");
            }
        }

        if let Some(val) = &filter.has_machines {
            if val == "true" {
                count_query.push_str(" AND EXISTS (SELECT 1 FROM machines m WHERE m.section_id = s.id)");
                data_query.push_str(" AND EXISTS (SELECT 1 FROM machines m WHERE m.section_id = s.id)");
            } else if val == "false" {
                count_query.push_str(" AND NOT EXISTS (SELECT 1 FROM machines m WHERE m.section_id = s.id)");
                data_query.push_str(" AND NOT EXISTS (SELECT 1 FROM machines m WHERE m.section_id = s.id)");
            }
        }

        if let Some(val) = &filter.has_users {
            if val == "true" {
                count_query.push_str(" AND EXISTS (SELECT 1 FROM user_sections us WHERE us.section_id = s.id)");
                data_query.push_str(" AND EXISTS (SELECT 1 FROM user_sections us WHERE us.section_id = s.id)");
            } else if val == "false" {
                count_query.push_str(" AND NOT EXISTS (SELECT 1 FROM user_sections us WHERE us.section_id = s.id)");
                data_query.push_str(" AND NOT EXISTS (SELECT 1 FROM user_sections us WHERE us.section_id = s.id)");
            }
        }

        if let Some(val) = &filter.has_order_types {
            if val == "true" {
                count_query.push_str(" AND EXISTS (SELECT 1 FROM section_order_types sot WHERE sot.section_id = s.id)");
                data_query.push_str(" AND EXISTS (SELECT 1 FROM section_order_types sot WHERE sot.section_id = s.id)");
            } else if val == "false" {
                count_query.push_str(" AND NOT EXISTS (SELECT 1 FROM section_order_types sot WHERE sot.section_id = s.id)");
                data_query.push_str(" AND NOT EXISTS (SELECT 1 FROM section_order_types sot WHERE sot.section_id = s.id)");
            }
        }

        let total_count: i32 = conn.query_row(&count_query, params_vec.as_slice(), |row| row.get(0))?;

        data_query.push_str(" GROUP BY s.id ORDER BY s.name");

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
            let id: i32 = row.get(0)?;
            let order_type_ids = Self::get_order_type_ids(conn, id)?;
            Ok(Section {
                id,
                name: row.get(1)?,
                machine_count: row.get(2)?,
                user_count: row.get(3)?,
                order_type_ids,
            })
        })?;

        let data = rows.collect::<Result<Vec<_>, _>>()?;

        Ok(FilterResponse {
            total_count,
            data,
        })
    }

    pub fn assign_order_types(conn: &Connection, section_id: i32, order_type_ids: Vec<i32>) -> Result<()> {
        conn.execute("DELETE FROM section_order_types WHERE section_id = ?1", params![section_id])?;
        
        for order_type_id in order_type_ids {
            conn.execute(
                "INSERT INTO section_order_types (section_id, order_type_id) VALUES (?1, ?2)",
                params![section_id, order_type_id],
            )?;
        }
        Ok(())
    }
}

