use chrono::Local;
use rusqlite::{params, Connection, Result};
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};

#[derive(Debug, Serialize)]
pub struct User {
    pub id: i32,
    pub full_name: String,
    pub staffid: String,
    pub password: String,
    pub phone_number: Option<String>,
    pub status: String,
    pub role_id: i32,
    pub plan_id: Option<i32>,
    pub created_at: String,
    pub updated_at: String,
    pub section_ids: Vec<i32>,
}

#[derive(Debug, Serialize)]
pub struct SignInResponse {
    pub id: i32,
    pub staffid: String,
    pub role: String,
    pub plan: String,
    pub whois: String,
}

impl User {
    fn fetch_sections(conn: &Connection, user_id: i32) -> Result<Vec<i32>> {
        let sections = conn.prepare("SELECT section_id FROM user_section WHERE user_id = ?1")?
            .query_map(params![user_id], |r| r.get(0))?.collect::<Result<Vec<_>, _>>()?;
        Ok(sections)
    }

    pub fn create(conn: &Connection, u: &UserCreatePayload) -> Result<User> {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        conn.execute(
            "INSERT INTO users (full_name, staffid, password, phone_number, status, role_id, plan_id, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![u.full_name, u.staffid, u.password, u.phone_number, u.status, u.role_id, u.plan_id, now, now],
        )?;
        let id = conn.last_insert_rowid() as i32;

        if let Some(sections) = &u.section_ids {
            for sec in sections {
                conn.execute("INSERT INTO user_section (user_id, section_id) VALUES (?1, ?2)", params![id, sec])?;
            }
        }

        let section_ids = Self::fetch_sections(conn, id)?;
        Ok(User {
            id,
            full_name: u.full_name.clone(),
            staffid: u.staffid.clone(),
            password: "******".to_string(),
            phone_number: u.phone_number.clone(),
            status: u.status.clone(),
            role_id: u.role_id,
            plan_id: u.plan_id,
            created_at: now.clone(),
            updated_at: now.clone(),
            section_ids,
        })
    }

    pub fn update(&mut self, conn: &Connection, u: &UserPayload) -> Result<()> {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        if let Some(full_name) = &u.full_name {
            conn.execute("UPDATE users SET full_name = ?1 WHERE id = ?2", params![full_name, self.id])?;
            self.full_name = full_name.clone();
        }
        if let Some(staffid) = &u.staffid {
            conn.execute("UPDATE users SET staffid = ?1 WHERE id = ?2", params![staffid, self.id])?;
            self.staffid = staffid.clone();
        }
        if let Some(password) = &u.password {
            let mut hasher = Sha256::new();
            hasher.update(password);
            let hashed_password = format!("{:x}", hasher.finalize());
            conn.execute("UPDATE users SET password = ?1 WHERE id = ?2", params![hashed_password, self.id])?;
            self.password = hashed_password;
        }
        if let Some(phone_number) = &u.phone_number {
            conn.execute("UPDATE users SET phone_number = ?1 WHERE id = ?2", params![phone_number, self.id])?;
            self.phone_number = Some(phone_number.clone());
        }
        if let Some(status) = &u.status {
            conn.execute("UPDATE users SET status = ?1 WHERE id = ?2", params![status, self.id])?;
            self.status = status.clone();
        }
        if let Some(role_id) = u.role_id {
            conn.execute("UPDATE users SET role_id = ?1 WHERE id = ?2", params![role_id, self.id])?;
            self.role_id = role_id;
        }
        if let Some(plan_id) = u.plan_id {
            conn.execute("UPDATE users SET plan_id = ?1 WHERE id = ?2", params![plan_id, self.id])?;
            self.plan_id = Some(plan_id);
        }
        if let Some(sections) = &u.section_ids {
            conn.execute("DELETE FROM user_section WHERE user_id = ?1", params![self.id])?;
            for sec in sections {
                conn.execute("INSERT INTO user_section (user_id, section_id) VALUES (?1, ?2)", params![self.id, sec])?;
            }
        }
        conn.execute("UPDATE users SET updated_at = ?1 WHERE id = ?2", params![now, self.id])?;
        self.updated_at = now;
        self.password = "******".to_string();
        self.section_ids = Self::fetch_sections(conn, self.id)?;
        Ok(())
    }

    pub fn delete(&self, conn: &Connection) -> Result<()> {
        conn.execute("DELETE FROM users WHERE id = ?1", params![self.id])?;
        Ok(())
    }

    pub fn find_by_id(conn: &Connection, id: i32) -> Result<Self> {
        let mut stmt = conn.prepare(
            "SELECT id, full_name, staffid, password, phone_number, status, role_id, plan_id, created_at, updated_at 
             FROM users WHERE id = ?1"
        )?;
        let user = stmt.query_row(params![id], |row| Ok(User {
            id: row.get(0)?,
            full_name: row.get(1)?,
            staffid: row.get(2)?,
            password: "******".to_string(),
            phone_number: row.get(4)?,
            status: row.get(5)?,
            role_id: row.get(6)?,
            plan_id: row.get(7)?,
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
            section_ids: vec![],
        }))?;
        let section_ids = Self::fetch_sections(conn, id)?;
        Ok(User { section_ids, ..user })
    }

    pub fn count_linked_records(conn: &Connection, user_id: i32) -> Result<i32> {
        let query = "SELECT COUNT(*) FROM weigh_log WHERE user_id = ?1";
        let mut stmt = conn.prepare(query)?;
        let count: i32 = stmt.query_row(params![user_id], |row| row.get(0))?;
        Ok(count)
    }

    pub fn all(conn: &Connection) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare(
            "SELECT id, full_name, staffid, password, phone_number, status, role_id, plan_id, created_at, updated_at FROM users ORDER BY created_at DESC"
        )?;
        let users = stmt.query_map([], |row| Ok(User {
            id: row.get(0)?,
            full_name: row.get(1)?,
            staffid: row.get(2)?,
            password: "******".to_string(),
            phone_number: row.get(4)?,
            status: row.get(5)?,
            role_id: row.get(6)?,
            plan_id: row.get(7)?,
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
            section_ids: vec![],
        }))?.collect::<Result<Vec<_>, _>>()?;
        users.into_iter().map(|u| {
            let section_ids = Self::fetch_sections(conn, u.id)?;
            Ok(User { section_ids, ..u })
        }).collect()
    }

    pub fn me(conn: &Connection, user_id: i32) -> Result<Self> {
        Self::find_by_id(conn, user_id)
    }

    pub fn signin(conn: &Connection, payload: &SigninPayload) -> Result<SignInResponse> {
        let mut hasher = Sha256::new();
        hasher.update(&payload.password);
        let hashed_password = format!("{:x}", hasher.finalize());
        let mut stmt = conn.prepare(
            "SELECT u.staffid, r.name AS role, p.name AS plan, u.id
            FROM users u
            LEFT JOIN roles r ON u.role_id = r.id
            LEFT JOIN plan p ON u.plan_id = p.id
            WHERE lower(u.staffid) = lower(?1) AND u.password = ?2"
        )?;
        let signin_response = stmt.query_row(params![&payload.staffid, &hashed_password], |row| {
            Ok(SignInResponse {
                staffid: row.get(0)?,
                role: row.get::<_, String>(1).unwrap_or_default(),
                plan: row.get::<_, String>(2).unwrap_or_default(),
                whois: format!(
                    "{} {} {}",
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(2).unwrap_or_default(),
                    row.get::<_, String>(1).unwrap_or_default()
                ),
                id: row.get(3)?,
            })
        })?;
        Ok(signin_response)
    }

}

#[derive(Deserialize)]
pub struct UserCreatePayload {
    pub full_name: String,
    pub staffid: String,
    pub password: Option<String>,
    pub phone_number: Option<String>,
    pub status: String,
    pub role_id: i32,
    pub plan_id: Option<i32>,
    pub section_ids: Option<Vec<i32>>,
}

#[derive(Deserialize)]
pub struct UserPayload {
    pub id: i32,
    pub full_name: Option<String>,
    pub staffid: Option<String>,
    pub password: Option<String>,
    pub phone_number: Option<String>,
    pub status: Option<String>,
    pub role_id: Option<i32>,
    pub plan_id: Option<i32>,
    pub section_ids: Option<Vec<i32>>,
}

#[derive(Deserialize)]
pub struct SigninPayload {
    pub staffid: String,
    pub password: String,
}
