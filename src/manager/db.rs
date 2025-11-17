use rusqlite::{Connection, Result};
use r2d2_sqlite::SqliteConnectionManager;
use r2d2::Pool;

pub fn init_local_db(path: &str) -> Result<()> {
    let conn = Connection::open(path)?;
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS content_type (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
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
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            full_name TEXT,
            staffid TEXT,
            phone_number TEXT,
            password TEXT,
            status TEXT,
            role_id INTEGER,
            created_at DATETIME,
            updated_at DATETIME,
            FOREIGN KEY (role_id) REFERENCES roles(id)
        );
        CREATE TABLE IF NOT EXISTS role_permissions (
            role_id INTEGER,
            permission_id INTEGER,
            PRIMARY KEY(role_id, permission_id),
            FOREIGN KEY(role_id) REFERENCES roles(id),
            FOREIGN KEY(permission_id) REFERENCES permissions(id)
        );

        CREATE TABLE IF NOT EXISTS shifts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT UNIQUE
        );
        CREATE TABLE IF NOT EXISTS colours (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT UNIQUE
        );
        CREATE TABLE IF NOT EXISTS solvent_types (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT UNIQUE
        );
        CREATE TABLE IF NOT EXISTS scrap_types (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT UNIQUE
        );
        CREATE TABLE IF NOT EXISTS downtime_reasons (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT UNIQUE
        );
        CREATE TABLE IF NOT EXISTS flag_reasons (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT UNIQUE
        );

        CREATE TABLE IF NOT EXISTS jobs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            shift_id INTEGER,
            production_order TEXT,
            batch_roll_no TEXT,
            start_weight DECIMAL(10,2),
            start_meter DECIMAL(10,2),
            created_by INTEGER,
            machine_id INTEGER,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (machine_id) REFERENCES machines(id),
            FOREIGN KEY (shift_id) REFERENCES shifts(id),
            FOREIGN KEY (created_by) REFERENCES users(id)
        );
        CREATE TABLE IF NOT EXISTS rolls (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            output_roll_no TEXT UNIQUE,
            final_meter DECIMAL(10,2),
            number_of_flags INTEGER DEFAULT 0,
            flag_reason_id INTEGER,
            final_weight DECIMAL(10,2),
            job_id INTEGER,
            created_by INTEGER,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (flag_reason_id) REFERENCES flag_reasons(id),
            FOREIGN KEY (job_id) REFERENCES jobs(id),
            FOREIGN KEY (created_by) REFERENCES users(id)
        );
        CREATE TABLE IF NOT EXISTS downtimes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            shift_id INTEGER,
            start_time DATETIME,
            end_time DATETIME,
            duration_minutes INTEGER,
            downtime_reason_id INTEGER,
            created_by INTEGER,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (shift_id) REFERENCES shifts(id),
            FOREIGN KEY (downtime_reason_id) REFERENCES downtime_reasons(id),
            FOREIGN KEY (created_by) REFERENCES users(id)
        );
        CREATE TABLE IF NOT EXISTS scraps (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            shift_id INTEGER,
            time DATETIME,
            scrap_type_id INTEGER,
            weight_kg DECIMAL(10,2),
            notes TEXT,
            created_by INTEGER,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (shift_id) REFERENCES shifts(id),
            FOREIGN KEY (scrap_type_id) REFERENCES scrap_types(id),
            FOREIGN KEY (created_by) REFERENCES users(id)
        );
        CREATE TABLE IF NOT EXISTS ink_usages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            shift_id INTEGER,
            colour_id INTEGER,
            batch_code TEXT,
            kgs_issued DECIMAL(10,2),
            created_by INTEGER,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (shift_id) REFERENCES shifts(id),
            FOREIGN KEY (colour_id) REFERENCES colours(id),
            FOREIGN KEY (created_by) REFERENCES users(id)
        );
        CREATE TABLE IF NOT EXISTS solvent_usages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            shift_id INTEGER,
            solvent_type_id INTEGER,
            kgs_issued DECIMAL(10,2),
            created_by INTEGER,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (shift_id) REFERENCES shifts(id),
            FOREIGN KEY (solvent_type_id) REFERENCES solvent_types(id),
            FOREIGN KEY (created_by) REFERENCES users(id)
        );
        CREATE TABLE IF NOT EXISTS machines (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT UNIQUE,
            label TEXT
        );
        CREATE TABLE IF NOT EXISTS user_machines (
            user_id INTEGER,
            machine_id INTEGER,
            PRIMARY KEY(user_id, machine_id),
            FOREIGN KEY(user_id) REFERENCES users(id),
            FOREIGN KEY(machine_id) REFERENCES machines(id)
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

