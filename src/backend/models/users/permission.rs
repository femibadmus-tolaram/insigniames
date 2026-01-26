use rusqlite::{Connection, OptionalExtension, Result, params, params_from_iter};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct Permission {
    pub id: i32,
    pub role_id: i32,
    pub model: String,
    pub can_create: bool,
    pub can_read: bool,
    pub can_update: bool,
    pub can_delete: bool,
}

#[derive(Deserialize)]
pub struct PermissionPayload {
    pub role_id: i32,
    pub model: String,
    pub can_create: bool,
    pub can_read: bool,
    pub can_update: bool,
    pub can_delete: bool,
}

#[derive(Deserialize)]
pub struct PermissionUpdatePayload {
    pub id: i32,
    pub can_create: Option<bool>,
    pub can_read: Option<bool>,
    pub can_update: Option<bool>,
    pub can_delete: Option<bool>,
}

impl Permission {
    pub fn ensure_admin_full_access(conn: &mut Connection) -> Result<()> {
        let tx = conn.transaction()?;
        tx.execute(
            "INSERT OR IGNORE INTO role_permissions (role_id, permission_id)
             SELECT 1, id FROM permissions",
            [],
        )?;
        tx.commit()?;
        Ok(())
    }
    pub fn create(conn: &Connection, p: &PermissionPayload) -> Result<Self> {
        conn.execute(
            "INSERT OR IGNORE INTO content_type (model) VALUES (?1)",
            params![p.model],
        )?;

        let content_type_id: i32 = conn.query_row(
            "SELECT id FROM content_type WHERE model = ?1",
            params![p.model],
            |row| row.get(0),
        )?;

        let mut stmt = conn.prepare(
            "SELECT p.id
             FROM permissions p
             JOIN role_permissions rp ON rp.permission_id = p.id
             WHERE rp.role_id = ?1 AND p.content_type_id = ?2",
        )?;
        let existing_ids: Vec<i32> = stmt
            .query_map(params![p.role_id, content_type_id], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;

        if !existing_ids.is_empty() {
            for id in &existing_ids {
                conn.execute(
                    "UPDATE permissions
                     SET can_create = ?1, can_read = ?2, can_update = ?3, can_delete = ?4
                     WHERE id = ?5",
                    params![p.can_create, p.can_read, p.can_update, p.can_delete, id],
                )?;
            }

            // Keep the newest permission row, remove duplicates.
            let keep_id = *existing_ids.iter().max().unwrap();
            let delete_ids: Vec<i32> = existing_ids
                .into_iter()
                .filter(|id| *id != keep_id)
                .collect();
            if !delete_ids.is_empty() {
                let placeholders = std::iter::repeat("?")
                    .take(delete_ids.len())
                    .collect::<Vec<_>>()
                    .join(",");
                let sql_rp = format!(
                    "DELETE FROM role_permissions WHERE role_id = ?1 AND permission_id IN ({})",
                    placeholders
                );
                let mut params_vec: Vec<&dyn rusqlite::ToSql> =
                    Vec::with_capacity(delete_ids.len() + 1);
                params_vec.push(&p.role_id);
                for id in &delete_ids {
                    params_vec.push(id);
                }
                conn.execute(&sql_rp, params_from_iter(params_vec))?;
            }

            return Ok(Permission {
                id: keep_id,
                role_id: p.role_id,
                model: p.model.clone(),
                can_create: p.can_create,
                can_read: p.can_read,
                can_update: p.can_update,
                can_delete: p.can_delete,
            });
        }

        conn.execute(
            "INSERT INTO permissions (content_type_id, can_create, can_read, can_update, can_delete)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                content_type_id,
                p.can_create,
                p.can_read,
                p.can_update,
                p.can_delete
            ],
        )?;

        let id = conn.last_insert_rowid() as i32;

        conn.execute(
            "INSERT OR IGNORE INTO role_permissions (role_id, permission_id) VALUES (?1, ?2)",
            params![p.role_id, id],
        )?;

        Ok(Permission {
            id,
            role_id: p.role_id,
            model: p.model.clone(),
            can_create: p.can_create,
            can_read: p.can_read,
            can_update: p.can_update,
            can_delete: p.can_delete,
        })
    }

    pub fn update(&mut self, conn: &Connection, p: &PermissionUpdatePayload) -> Result<()> {
        if let Some(can_create) = p.can_create {
            conn.execute(
                "UPDATE permissions SET can_create = ?1 WHERE id = ?2",
                params![can_create, self.id],
            )?;
            self.can_create = can_create;
        }
        if let Some(can_read) = p.can_read {
            conn.execute(
                "UPDATE permissions SET can_read = ?1 WHERE id = ?2",
                params![can_read, self.id],
            )?;
            self.can_read = can_read;
        }
        if let Some(can_update) = p.can_update {
            conn.execute(
                "UPDATE permissions SET can_update = ?1 WHERE id = ?2",
                params![can_update, self.id],
            )?;
            self.can_update = can_update;
        }
        if let Some(can_delete) = p.can_delete {
            conn.execute(
                "UPDATE permissions SET can_delete = ?1 WHERE id = ?2",
                params![can_delete, self.id],
            )?;
            self.can_delete = can_delete;
        }
        Ok(())
    }

    pub fn delete(conn: &Connection, id: i32) -> Result<()> {
        conn.execute(
            "DELETE FROM role_permissions WHERE permission_id = ?1",
            params![id],
        )?;
        conn.execute("DELETE FROM permissions WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn count_linked_roles(conn: &Connection, permission_id: i32) -> Result<i32> {
        let mut stmt =
            conn.prepare("SELECT COUNT(*) FROM role_permissions WHERE permission_id = ?1")?;
        let mut rows = stmt.query(params![permission_id])?;
        if let Some(row) = rows.next()? {
            let count: i32 = row.get(0)?;
            Ok(count)
        } else {
            Ok(0)
        }
    }

    pub fn all_for_user(conn: &Connection, user_id: i32) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare(
            "SELECT p.id, rp.role_id, c.model, 
                    p.can_create, p.can_read, p.can_update, p.can_delete
            FROM permissions p
            JOIN content_type c ON p.content_type_id = c.id
            JOIN role_permissions rp ON p.id = rp.permission_id
            JOIN users u ON u.role_id = rp.role_id
            WHERE u.id = ?1",
        )?;

        let rows = stmt.query_map([user_id], |row| {
            Ok(Permission {
                id: row.get(0)?,
                role_id: row.get(1)?,
                model: row.get(2)?,
                can_create: row.get(3)?,
                can_read: row.get(4)?,
                can_update: row.get(5)?,
                can_delete: row.get(6)?,
            })
        })?;

        rows.collect()
    }

    pub fn find_by_id(conn: &Connection, id: i32) -> Result<bool> {
        let mut stmt = conn.prepare("SELECT 1 FROM permissions WHERE id = ?1")?;
        let result: Option<i32> = stmt.query_row(params![id], |row| row.get(0)).optional()?;
        Ok(result.is_some())
    }

    pub fn all(conn: &mut Connection) -> Result<Vec<Self>> {
        let _ = Self::cleanup_role_permission_duplicates(conn);
        let _ = Self::ensure_admin_full_access(conn);
        let mut stmt = conn.prepare(
            "SELECT p.id, rp.role_id, c.model, 
                    p.can_create, p.can_read, p.can_update, p.can_delete
             FROM permissions p
             JOIN content_type c ON p.content_type_id = c.id
             JOIN role_permissions rp ON p.id = rp.permission_id",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(Permission {
                id: row.get(0)?,
                role_id: row.get(1)?,
                model: row.get(2)?,
                can_create: row.get(3)?,
                can_read: row.get(4)?,
                can_update: row.get(5)?,
                can_delete: row.get(6)?,
            })
        })?;

        rows.collect()
    }

    pub fn cleanup_role_permission_duplicates(conn: &mut Connection) -> Result<usize> {
        let tx = conn.transaction()?;

        let mut map: std::collections::HashMap<(i32, i32), Vec<i32>> =
            std::collections::HashMap::new();
        {
            let mut stmt = tx.prepare(
                "SELECT rp.role_id, p.content_type_id, p.id
                 FROM role_permissions rp
                 JOIN permissions p ON rp.permission_id = p.id
                 ORDER BY rp.role_id ASC, p.content_type_id ASC, p.id ASC",
            )?;

            let rows = stmt.query_map([], |row| {
                Ok((
                    row.get::<_, i32>(0)?,
                    row.get::<_, i32>(1)?,
                    row.get::<_, i32>(2)?,
                ))
            })?;

            for row in rows {
                let (role_id, content_type_id, perm_id) = row?;
                map.entry((role_id, content_type_id))
                    .or_default()
                    .push(perm_id);
            }
        }

        let mut removed = 0usize;
        for ((role_id, _content_type_id), mut ids) in map {
            if ids.len() <= 1 {
                continue;
            }
            ids.sort_unstable();
            let keep_id = *ids.last().unwrap();
            let delete_ids: Vec<i32> = ids.into_iter().filter(|id| *id != keep_id).collect();
            if delete_ids.is_empty() {
                continue;
            }
            let placeholders = std::iter::repeat("?")
                .take(delete_ids.len())
                .collect::<Vec<_>>()
                .join(",");
            let sql = format!(
                "DELETE FROM role_permissions WHERE role_id = ?1 AND permission_id IN ({})",
                placeholders
            );
            let mut params_vec: Vec<&dyn rusqlite::ToSql> =
                Vec::with_capacity(delete_ids.len() + 1);
            params_vec.push(&role_id);
            for id in &delete_ids {
                params_vec.push(id);
            }
            removed += tx.execute(&sql, params_from_iter(params_vec))? as usize;
        }

        tx.commit()?;
        Ok(removed)
    }
}
