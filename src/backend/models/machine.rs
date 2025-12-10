use rusqlite::{params, Connection, Result};
use serde::{Serialize, Deserialize};
use crate::backend::models::FilterResponse;

#[derive(Debug, Serialize)]
pub struct Machine {
    pub id: i32,
    pub name: String,
    pub label: String,
    pub section_id: i32,
    pub section_name: String,
    pub section_order_types: Option<String>,
    pub user_count: i32,
    pub job_count: i32,
}

#[derive(Debug, Serialize)]
pub struct MachineStats {
    pub total_machines: i32,
    pub total_users: i32,
    pub total_jobs: i32,
    pub total_sections: i32,
}

#[derive(Deserialize)]
pub struct MachineCreatePayload {
    pub name: String,
    pub label: String,
    pub section_id: i32,
}

#[derive(Deserialize)]
pub struct MachinePayload {
    pub id: i32,
    pub name: Option<String>,
    pub label: Option<String>,
    pub section_id: Option<i32>,
}

#[derive(Deserialize)]
pub struct MachineFilterPayload {
    pub name: Option<String>,
    pub label: Option<String>,
    pub section_id: Option<String>,
    pub has_users: Option<String>,
    pub has_jobs: Option<String>,
    pub per_page: Option<String>,
    pub page: Option<String>,
    pub user_id: Option<String>,
}

impl Machine {
    pub fn create(conn: &Connection, data: &MachineCreatePayload) -> Result<Self> {
        conn.execute(
            "INSERT INTO machines (name, label, section_id) VALUES (?1, ?2, ?3)",
            params![data.name, data.label, data.section_id],
        )?;
        let id = conn.last_insert_rowid() as i32;
        
        let mut stmt = conn.prepare(
            "SELECT m.*, s.name as section_name,
             mot.name as section_order_types,
             COUNT(DISTINCT us.user_id) as user_count,
             COUNT(DISTINCT j.id) as job_count
             FROM machines m
             JOIN sections s ON m.section_id = s.id
             LEFT JOIN manufacturing_order_types mot ON s.order_type_id = mot.id
             LEFT JOIN user_sections us ON m.section_id = us.section_id
             LEFT JOIN jobs j ON m.id = j.machine_id
             WHERE m.id = ?1
             GROUP BY m.id"
        )?;
        
        stmt.query_row(params![id], |row| Ok(Machine {
            id: row.get(0)?,
            name: row.get(1)?,
            label: row.get(2)?,
            section_id: row.get(3)?,
            section_name: row.get(4)?,
            section_order_types: row.get(5)?,
            user_count: row.get(6)?,
            job_count: row.get(7)?,
        }))
    }

    pub fn update(&mut self, conn: &Connection, data: &MachinePayload) -> Result<()> {
        if let Some(name) = &data.name {
            conn.execute("UPDATE machines SET name = ?1 WHERE id = ?2", params![name, self.id])?;
            self.name = name.clone();
        }
        if let Some(label) = &data.label {
            conn.execute("UPDATE machines SET label = ?1 WHERE id = ?2", params![label, self.id])?;
            self.label = label.clone();
        }
        if let Some(section_id) = &data.section_id {
            conn.execute("UPDATE machines SET section_id = ?1 WHERE id = ?2", params![section_id, self.id])?;
            self.section_id = *section_id;
            
            let mut stmt = conn.prepare(
                "SELECT s.name as section_name,
                 mot.name as section_order_types
                 FROM sections s
                 LEFT JOIN manufacturing_order_types mot ON s.order_type_id = mot.id
                 WHERE s.id = ?1"
            )?;
            
            stmt.query_row(params![section_id], |row| {
                self.section_name = row.get(0)?;
                self.section_order_types = row.get(1)?;
                Ok(())
            })?;
        }
        Ok(())
    }

    pub fn delete(&self, conn: &Connection) -> Result<()> {
        conn.execute("DELETE FROM jobs WHERE machine_id = ?1", params![self.id])?;
        conn.execute("DELETE FROM machines WHERE id = ?1", params![self.id])?;
        Ok(())
    }

    pub fn has_jobs(conn: &Connection, machine_id: i32) -> Result<bool> {
        let count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM jobs WHERE machine_id = ?1",
            params![machine_id],
            |row| row.get(0)
        )?;
        Ok(count > 0)
    }

    pub fn find_by_id(conn: &Connection, id: i32) -> Result<Self> {
        let mut stmt = conn.prepare(
            "SELECT m.*, s.name as section_name,
             mot.name as section_order_types,
             COUNT(DISTINCT us.user_id) as user_count,
             COUNT(DISTINCT j.id) as job_count
             FROM machines m
             JOIN sections s ON m.section_id = s.id
             LEFT JOIN manufacturing_order_types mot ON s.order_type_id = mot.id
             LEFT JOIN user_sections us ON m.section_id = us.section_id
             LEFT JOIN jobs j ON m.id = j.machine_id
             WHERE m.id = ?1
             GROUP BY m.id"
        )?;
        stmt.query_row(params![id], |row| Ok(Machine {
            id: row.get(0)?,
            name: row.get(1)?,
            label: row.get(2)?,
            section_id: row.get(3)?,
            section_name: row.get(4)?,
            section_order_types: row.get(5)?,
            user_count: row.get(6)?,
            job_count: row.get(7)?,
        }))
    }

    pub fn all(conn: &Connection) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare(
            "SELECT m.*, s.name as section_name,
             mot.name as section_order_types,
             COUNT(DISTINCT us.user_id) as user_count,
             COUNT(DISTINCT j.id) as job_count
             FROM machines m
             JOIN sections s ON m.section_id = s.id
             LEFT JOIN manufacturing_order_types mot ON s.order_type_id = mot.id
             LEFT JOIN user_sections us ON m.section_id = us.section_id
             LEFT JOIN jobs j ON m.id = j.machine_id
             GROUP BY m.id
             ORDER BY s.name, m.name"
        )?;
        let machines = stmt.query_map([], |row| Ok(Machine {
            id: row.get(0)?,
            name: row.get(1)?,
            label: row.get(2)?,
            section_id: row.get(3)?,
            section_name: row.get(4)?,
            section_order_types: row.get(5)?,
            user_count: row.get(6)?,
            job_count: row.get(7)?,
        }))?.collect::<Result<Vec<_>, _>>()?;
        Ok(machines)
    }

    pub fn get_stats(conn: &Connection) -> Result<MachineStats> {
        let total_machines: i32 = conn.query_row("SELECT COUNT(*) FROM machines", [], |row| row.get(0))?;
        let total_users: i32 = conn.query_row("SELECT COUNT(DISTINCT user_id) FROM user_sections", [], |row| row.get(0))?;
        let total_jobs: i32 = conn.query_row("SELECT COUNT(*) FROM jobs", [], |row| row.get(0))?;
        let total_sections: i32 = conn.query_row("SELECT COUNT(*) FROM sections", [], |row| row.get(0))?;

        Ok(MachineStats {
            total_machines,
            total_users,
            total_jobs,
            total_sections,
        })
    }

    pub fn filter(conn: &Connection, filter: &MachineFilterPayload) -> Result<FilterResponse<Self>> {
        let mut count_query = "SELECT COUNT(DISTINCT m.id) FROM machines m JOIN sections s ON m.section_id = s.id WHERE 1=1".to_string();
        let mut data_query = "SELECT m.*, s.name as section_name,
                            mot.name as section_order_types,
                            COUNT(DISTINCT us.user_id) as user_count,
                            COUNT(DISTINCT j.id) as job_count
                            FROM machines m
                            JOIN sections s ON m.section_id = s.id
                            LEFT JOIN manufacturing_order_types mot ON s.order_type_id = mot.id
                            LEFT JOIN user_sections us ON m.section_id = us.section_id
                            LEFT JOIN jobs j ON m.id = j.machine_id
                            WHERE 1=1".to_string();
        let mut params_vec: Vec<&dyn rusqlite::ToSql> = vec![];

        let mut names: Vec<String> = vec![];
        let mut labels: Vec<String> = vec![];
        let mut user_ids: Vec<i32> = vec![];
        let mut section_ids: Vec<i32> = vec![];
        let mut pages: Vec<i32> = vec![];
        let mut per_pages: Vec<i32> = vec![];

        if let Some(val) = &filter.name {
            if !val.is_empty() {
                names.push(format!("%{}%", val));
                params_vec.push(names.last().unwrap());
                count_query.push_str(" AND m.name LIKE ?");
                data_query.push_str(" AND m.name LIKE ?");
            }
        }

        if let Some(val) = &filter.label {
            if !val.is_empty() {
                labels.push(format!("%{}%", val));
                params_vec.push(labels.last().unwrap());
                count_query.push_str(" AND m.label LIKE ?");
                data_query.push_str(" AND m.label LIKE ?");
            }
        }

        if let Some(val) = &filter.section_id {
            if !val.is_empty() {
                if let Ok(section_id) = val.parse::<i32>() {
                    section_ids.push(section_id);
                    params_vec.push(section_ids.last().unwrap());
                    count_query.push_str(" AND m.section_id = ?");
                    data_query.push_str(" AND m.section_id = ?");
                }
            }
        }

        if let Some(val) = &filter.user_id {
            if !val.is_empty() {
                if let Ok(user_id) = val.parse::<i32>() {
                    user_ids.push(user_id);
                    params_vec.push(user_ids.last().unwrap());
                    count_query.push_str(" AND EXISTS (SELECT 1 FROM user_sections us WHERE us.section_id = m.section_id AND us.user_id = ?)");
                    data_query.push_str(" AND EXISTS (SELECT 1 FROM user_sections us WHERE us.section_id = m.section_id AND us.user_id = ?)");
                }
            }
        }

        if let Some(val) = &filter.has_users {
            if val == "true" {
                count_query.push_str(" AND EXISTS (SELECT 1 FROM user_sections us WHERE us.section_id = m.section_id)");
                data_query.push_str(" AND EXISTS (SELECT 1 FROM user_sections us WHERE us.section_id = m.section_id)");
            } else if val == "false" {
                count_query.push_str(" AND NOT EXISTS (SELECT 1 FROM user_sections us WHERE us.section_id = m.section_id)");
                data_query.push_str(" AND NOT EXISTS (SELECT 1 FROM user_sections us WHERE us.section_id = m.section_id)");
            }
        }

        if let Some(val) = &filter.has_jobs {
            if val == "true" {
                count_query.push_str(" AND EXISTS (SELECT 1 FROM jobs j WHERE j.machine_id = m.id)");
                data_query.push_str(" AND EXISTS (SELECT 1 FROM jobs j WHERE j.machine_id = m.id)");
            } else if val == "false" {
                count_query.push_str(" AND NOT EXISTS (SELECT 1 FROM jobs j WHERE j.machine_id = m.id)");
                data_query.push_str(" AND NOT EXISTS (SELECT 1 FROM jobs j WHERE j.machine_id = m.id)");
            }
        }

        let total_count: i32 = conn.query_row(&count_query, params_vec.as_slice(), |row| row.get(0))?;

        data_query.push_str(" GROUP BY m.id ORDER BY s.name, m.name");

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
            Ok(Machine {
                id: row.get(0)?,
                name: row.get(1)?,
                label: row.get(2)?,
                section_id: row.get(3)?,
                section_name: row.get(4)?,
                section_order_types: row.get(5)?,
                user_count: row.get(6)?,
                job_count: row.get(7)?,
            })
        })?;

        let data = rows.collect::<Result<Vec<_>, _>>()?;

        Ok(FilterResponse {
            total_count,
            data,
        })
    }
}

