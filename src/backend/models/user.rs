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
    pub created_at: String,
    pub updated_at: String,
    pub machine_ids: Vec<i32>,
}

#[derive(Debug, Serialize)]
pub struct SignInResponse {
    pub id: i32,
    pub staffid: String,
    pub role: String,
    pub whois: String,
}

#[derive(Deserialize)]
pub struct UserFilterPayload {
    pub full_name: Option<String>,
    pub staffid: Option<String>,
    pub status: Option<String>,
    pub role_id: Option<String>,
    pub machine_id: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub per_page: Option<String>,
    pub page: Option<String>,
}

impl User {
    pub fn staffid_exists(conn: &Connection, staffid: &str) -> Result<bool> {
        let count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM users WHERE staffid = ?1",
            params![staffid],
            |row| row.get(0)
        )?;
        Ok(count > 0)
    }

    pub fn count_linked_records(conn: &Connection, user_id: i32) -> Result<i32> {
        let mut total_count = 0;

        let jobs_count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM jobs WHERE created_by = ?1",
            params![user_id],
            |row| row.get(0),
        )?;
        total_count += jobs_count;

        let rolls_count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM rolls WHERE created_by = ?1",
            params![user_id],
            |row| row.get(0),
        )?;
        total_count += rolls_count;

        let downtimes_count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM downtimes WHERE created_by = ?1",
            params![user_id],
            |row| row.get(0),
        )?;
        total_count += downtimes_count;

        let scraps_count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM scraps WHERE created_by = ?1",
            params![user_id],
            |row| row.get(0),
        )?;
        total_count += scraps_count;

        let ink_usages_count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM ink_usages WHERE created_by = ?1",
            params![user_id],
            |row| row.get(0),
        )?;
        total_count += ink_usages_count;

        let solvent_usages_count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM solvent_usages WHERE created_by = ?1",
            params![user_id],
            |row| row.get(0),
        )?;
        total_count += solvent_usages_count;

        Ok(total_count)
    }

    pub fn create(conn: &Connection, u: &UserCreatePayload) -> Result<User> {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        conn.execute(
            "INSERT INTO users (full_name, staffid, password, phone_number, status, role_id, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![u.full_name, u.staffid, u.password, u.phone_number, u.status, u.role_id, now, now],
        )?;
        let id = conn.last_insert_rowid() as i32;

        let mut machine_ids = Vec::new();
        if let Some(machine_ids_payload) = &u.machine_ids {
            for &machine_id in machine_ids_payload {
                conn.execute(
                    "INSERT INTO user_machines (user_id, machine_id) VALUES (?1, ?2)",
                    params![id, machine_id],
                )?;
                machine_ids.push(machine_id);
            }
        }

        Ok(User {
            id,
            full_name: u.full_name.clone(),
            staffid: u.staffid.clone(),
            password: "******".to_string(),
            phone_number: u.phone_number.clone(),
            status: u.status.clone(),
            role_id: u.role_id,
            created_at: now.clone(),
            updated_at: now.clone(),
            machine_ids,
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
        
        if let Some(machine_ids) = &u.machine_ids {
            conn.execute("DELETE FROM user_machines WHERE user_id = ?1", params![self.id])?;
            
            for &machine_id in machine_ids {
                conn.execute(
                    "INSERT INTO user_machines (user_id, machine_id) VALUES (?1, ?2)",
                    params![self.id, machine_id],
                )?;
            }
            
            self.machine_ids = machine_ids.clone();
        }
        
        conn.execute("UPDATE users SET updated_at = ?1 WHERE id = ?2", params![now, self.id])?;
        self.updated_at = now;
        self.password = "******".to_string();
        Ok(())
    }

    pub fn delete(&self, conn: &Connection) -> Result<()> {
        conn.execute("DELETE FROM user_machines WHERE user_id = ?1", params![self.id])?;
        conn.execute("DELETE FROM users WHERE id = ?1", params![self.id])?;
        Ok(())
    }

    pub fn find_by_id(conn: &Connection, id: i32) -> Result<Self> {
        let mut stmt = conn.prepare(
            "SELECT u.id, u.full_name, u.staffid, u.password, u.phone_number, u.status, u.role_id, u.created_at, u.updated_at
             FROM users u
             WHERE u.id = ?1"
        )?;
        
        let mut user = stmt.query_row(params![id], |row| Ok(User {
            id: row.get(0)?,
            full_name: row.get(1)?,
            staffid: row.get(2)?,
            password: "******".to_string(),
            phone_number: row.get(4)?,
            status: row.get(5)?,
            role_id: row.get(6)?,
            created_at: row.get(7)?,
            updated_at: row.get(8)?,
            machine_ids: Vec::new(),
        }))?;
        
        user.machine_ids = user.get_machines(conn)?;
        Ok(user)
    }

    pub fn all(conn: &Connection) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare(
            "SELECT u.id, u.full_name, u.staffid, u.password, u.phone_number, u.status, u.role_id, u.created_at, u.updated_at
             FROM users u
             ORDER BY u.created_at DESC"
        )?;
        
        let mut users = stmt.query_map([], |row| Ok(User {
            id: row.get(0)?,
            full_name: row.get(1)?,
            staffid: row.get(2)?,
            password: "******".to_string(),
            phone_number: row.get(4)?,
            status: row.get(5)?,
            role_id: row.get(6)?,
            created_at: row.get(7)?,
            updated_at: row.get(8)?,
            machine_ids: Vec::new(),
        }))?.collect::<Result<Vec<_>, _>>()?;
        
        for user in &mut users {
            user.machine_ids = user.get_machines(conn)?;
        }
        
        Ok(users)
    }

    pub fn me(conn: &Connection, user_id: i32) -> Result<Self> {
        Self::find_by_id(conn, user_id)
    }

    pub fn signin(conn: &Connection, payload: &SigninPayload) -> Result<SignInResponse> {
        let mut hasher = Sha256::new();
        hasher.update(&payload.password);
        let hashed_password = format!("{:x}", hasher.finalize());
        let mut stmt = conn.prepare(
            "SELECT u.staffid, r.name AS role, u.id
            FROM users u
            LEFT JOIN roles r ON u.role_id = r.id
            WHERE lower(u.staffid) = lower(?1) AND u.password = ?2"
        )?;
        let signin_response = stmt.query_row(params![&payload.staffid, &hashed_password], |row| {
            Ok(SignInResponse {
                staffid: row.get(0)?,
                role: row.get::<_, String>(1).unwrap_or_default(),
                whois: format!(
                    "{} {}",
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1).unwrap_or_default()
                ),
                id: row.get(2)?,
            })
        })?;
        Ok(signin_response)
    }

    pub fn add_machine(&self, conn: &Connection, machine_id: i32) -> Result<()> {
        conn.execute(
            "INSERT OR IGNORE INTO user_machines (user_id, machine_id) VALUES (?1, ?2)",
            params![self.id, machine_id],
        )?;
        Ok(())
    }

    pub fn remove_machine(&self, conn: &Connection, machine_id: i32) -> Result<()> {
        conn.execute(
            "DELETE FROM user_machines WHERE user_id = ?1 AND machine_id = ?2",
            params![self.id, machine_id],
        )?;
        Ok(())
    }

    pub fn get_machines(&self, conn: &Connection) -> Result<Vec<i32>> {
        let mut stmt = conn.prepare(
            "SELECT machine_id FROM user_machines WHERE user_id = ?1"
        )?;
        let machine_ids = stmt.query_map(params![self.id], |row| {
            row.get(0)
        })?.collect::<Result<Vec<i32>, _>>()?;
        Ok(machine_ids)
    }

    pub fn filter(conn: &Connection, filter: &UserFilterPayload) -> Result<Vec<Self>> {
        let mut query = "SELECT u.id, u.full_name, u.staffid, u.password, u.phone_number, u.status, u.role_id, u.created_at, u.updated_at
                        FROM users u
                        WHERE 1=1".to_string();
        let mut params_vec: Vec<&dyn rusqlite::ToSql> = vec![];

        let mut full_names: Vec<String> = vec![];
        let mut staffids: Vec<String> = vec![];
        let mut statuses: Vec<String> = vec![];
        let mut role_ids: Vec<i32> = vec![];
        let mut machine_ids: Vec<i32> = vec![];
        let mut start_dates: Vec<String> = vec![];
        let mut end_dates: Vec<String> = vec![];
        let mut pages: Vec<i32> = vec![];
        let mut per_pages: Vec<i32> = vec![];

        if let Some(val) = &filter.full_name {
            if !val.is_empty() {
                full_names.push(format!("%{}%", val));
                params_vec.push(full_names.last().unwrap());
                query.push_str(" AND u.full_name LIKE ?");
            }
        }

        if let Some(val) = &filter.staffid {
            if !val.is_empty() {
                staffids.push(format!("%{}%", val));
                params_vec.push(staffids.last().unwrap());
                query.push_str(" AND u.staffid LIKE ?");
            }
        }

        if let Some(val) = &filter.status {
            if !val.is_empty() {
                statuses.push(val.clone());
                params_vec.push(statuses.last().unwrap());
                query.push_str(" AND u.status = ?");
            }
        }

        if let Some(val) = &filter.role_id {
            if let Ok(parsed) = val.parse::<i32>() {
                role_ids.push(parsed);
                params_vec.push(role_ids.last().unwrap());
                query.push_str(" AND u.role_id = ?");
            }
        }

        if let Some(val) = &filter.machine_id {
            if let Ok(parsed) = val.parse::<i32>() {
                machine_ids.push(parsed);
                params_vec.push(machine_ids.last().unwrap());
                query.push_str(" AND u.id IN (SELECT user_id FROM user_machines WHERE machine_id = ?)");
            }
        }

        if let Some(val) = &filter.start_date {
            if !val.is_empty() {
                start_dates.push(val.clone());
                params_vec.push(start_dates.last().unwrap());
                query.push_str(" AND date(u.created_at) >= date(?)");
            }
        }

        if let Some(val) = &filter.end_date {
            if !val.is_empty() {
                end_dates.push(val.clone());
                params_vec.push(end_dates.last().unwrap());
                query.push_str(" AND date(u.created_at) <= date(?)");
            }
        }

        if let (Some(page), Some(per_page)) = (&filter.page, &filter.per_page) {
            if let (Ok(page_val), Ok(per_page_val)) = (page.parse::<i32>(), per_page.parse::<i32>()) {
                if per_page_val > 0 {
                    let offset = (page_val - 1) * per_page_val;
                    pages.push(offset);
                    per_pages.push(per_page_val);
                    query.push_str(" ORDER BY u.created_at DESC LIMIT ? OFFSET ?");
                    params_vec.push(per_pages.last().unwrap());
                    params_vec.push(pages.last().unwrap());
                }
            }
        } else {
            query.push_str(" ORDER BY u.created_at DESC");
        }

        let mut stmt = conn.prepare(&query)?;
        let mut users = stmt.query_map(params_vec.as_slice(), |row| {
            Ok(User {
                id: row.get(0)?,
                full_name: row.get(1)?,
                staffid: row.get(2)?,
                password: "******".to_string(),
                phone_number: row.get(4)?,
                status: row.get(5)?,
                role_id: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
                machine_ids: Vec::new(),
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        
        for user in &mut users {
            user.machine_ids = user.get_machines(conn)?;
        }

        Ok(users)
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
    pub machine_ids: Option<Vec<i32>>,
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
    pub machine_ids: Option<Vec<i32>>,
}

#[derive(Deserialize)]
pub struct SigninPayload {
    pub staffid: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct UserMachinePayload {
    pub user_id: i32,
    pub machine_id: i32,
}