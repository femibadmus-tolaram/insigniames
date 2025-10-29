use rusqlite::{Connection, Result};
use r2d2_sqlite::SqliteConnectionManager;
use r2d2::Pool;

pub fn init_local_db(path: &str) -> Result<()> {
    let conn = Connection::open(path)?;
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS content_type (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            app_label TEXT,
            model TEXT
        );
        CREATE TABLE IF NOT EXISTS permissions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            codename TEXT UNIQUE,
            name TEXT,
            content_type_id INTEGER,
            can_create BOOLEAN DEFAULT 0,
            can_read BOOLEAN DEFAULT 0,
            can_update BOOLEAN DEFAULT 0,
            can_delete BOOLEAN DEFAULT 0,
            FOREIGN KEY (content_type_id) REFERENCES content_type(id)
        );
        CREATE TABLE IF NOT EXISTS roles (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT UNIQUE,
            description TEXT
        );
        CREATE TABLE IF NOT EXISTS plan (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT,
            status TEXT
        );
        CREATE TABLE IF NOT EXISTS sections (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT,
            status TEXT,
            plan_id INTEGER,
            FOREIGN KEY (plan_id) REFERENCES plan(id)
        );
        CREATE TABLE IF NOT EXISTS contractors (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT,
            status TEXT,
            section_id INTEGER,
            FOREIGN KEY (section_id) REFERENCES sections(id)
        );
        CREATE TABLE IF NOT EXISTS sku (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT,
            color TEXT,
            sku_code TEXT,
            category TEXT,
            description TEXT,
            product TEXT,
            lsl REAL,
            usl REAL,
            grammage REAL,
            sku_type TEXT
        );
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            full_name TEXT,
            staffid TEXT,
            phone_number TEXT,
            password TEXT,
            status TEXT,
            role_id INTEGER,
            plan_id INTEGER,
            created_at DATETIME,
            updated_at DATETIME,
            FOREIGN KEY (role_id) REFERENCES roles(id),
            FOREIGN KEY (plan_id) REFERENCES plan(id)
        );
        CREATE TABLE IF NOT EXISTS schedule (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            process_order TEXT,
            date DATETIME,
            sku_id INTEGER,
            section_id INTEGER,
            plan_id INTEGER,
            shift TEXT,
            FOREIGN KEY (sku_id) REFERENCES sku(id),
            FOREIGN KEY (section_id) REFERENCES sections(id),
            FOREIGN KEY (plan_id) REFERENCES plan(id)
        );
        CREATE TABLE IF NOT EXISTS role_permissions (
            role_id INTEGER,
            permission_id INTEGER,
            PRIMARY KEY(role_id, permission_id),
            FOREIGN KEY(role_id) REFERENCES roles(id),
            FOREIGN KEY(permission_id) REFERENCES permissions(id)
        );
        CREATE TABLE IF NOT EXISTS user_section (
            user_id INTEGER,
            section_id INTEGER,
            PRIMARY KEY (user_id, section_id),
            FOREIGN KEY (user_id) REFERENCES users(id),
            FOREIGN KEY (section_id) REFERENCES sections(id)
        );
        CREATE TABLE IF NOT EXISTS weigh_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            schedule_id INTEGER,
            contractor_id INTEGER,
            section_id INTEGER,
            plan_id INTEGER,
            user_id INTEGER,
            created_at DATETIME,
            status TEXT,
            code TEXT,
            weight TEXT,
            FOREIGN KEY (schedule_id) REFERENCES schedule(id),
            FOREIGN KEY (contractor_id) REFERENCES contractors(id),
            FOREIGN KEY (section_id) REFERENCES sections(id),
            FOREIGN KEY (plan_id) REFERENCES plan(id),
            FOREIGN KEY (user_id) REFERENCES users(id)
        );
        ",
    )?;
    Ok(())
}

pub fn connect_local_db(path: &str) -> Result<Pool<SqliteConnectionManager>> {
    let manager = SqliteConnectionManager::file(path);
    let pool = Pool::new(manager).unwrap();
    Ok(pool)
}

