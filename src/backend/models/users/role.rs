use rusqlite::{params, Connection, Result};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize)]
pub struct Role {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
}

impl Role {
    pub fn create(conn: &Connection, r: &RolePayload) -> Result<Self> {
        conn.execute(
            "INSERT INTO roles (name, description) VALUES (?1, ?2)",
            params![r.name, r.description],
        )?;
        let id = conn.last_insert_rowid() as i32;
        Ok(Role { id, name: r.name.clone(), description: r.description.clone() })
    }

    pub fn update(&mut self, conn: &Connection, r: &RoleUpdatePayload) -> Result<()> {
        if let Some(name) = &r.name {
            conn.execute("UPDATE roles SET name = ?1 WHERE id = ?2", params![name, self.id])?;
            self.name = name.clone();
        }
        if let Some(description) = &r.description {
            conn.execute("UPDATE roles SET description = ?1 WHERE id = ?2", params![description, self.id])?;
            self.description = Some(description.clone());
        }
        Ok(())
    }

    pub fn delete(&self, conn: &Connection) -> Result<()> {
        conn.execute("DELETE FROM roles WHERE id = ?1", params![self.id])?;
        Ok(())
    }

    pub fn count_linked_records(conn: &Connection, role_id: i32) -> Result<i32> {
        let query = "
            SELECT 
                (SELECT COUNT(*) FROM users WHERE role_id = ?1) +
                (SELECT COUNT(*) FROM role_permissions WHERE role_id = ?1)
        ";

        let mut stmt = conn.prepare(query)?;
        let count: i32 = stmt.query_row(params![role_id], |row| row.get(0))?;
        Ok(count)
    }

    pub fn find_by_id(conn: &Connection, id: i32) -> Result<Self> {
        conn.query_row(
            "SELECT id, name, description FROM roles WHERE id = ?1",
            params![id],
            |row| Ok(Role { id: row.get(0)?, name: row.get(1)?, description: row.get(2)? }),
        )
    }

    pub fn all(conn: &Connection) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare("SELECT id, name, description FROM roles ORDER BY id DESC")?;
        let rows = stmt.query_map([], |row| Ok(Role { id: row.get(0)?, name: row.get(1)?, description: row.get(2)? }))?;
        rows.collect()
    }
}

#[derive(Deserialize)]
pub struct RolePayload {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Deserialize)]
pub struct RoleUpdatePayload {
    pub id: i32,
    pub name: Option<String>,
    pub description: Option<String>,
}

