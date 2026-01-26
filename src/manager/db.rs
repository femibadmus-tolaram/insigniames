use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{Connection, Result};

pub fn init_local_db(path: &str) -> Result<()> {
    let conn = Connection::open(path)?;
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS content_type (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            model TEXT UNIQUE
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
            page_id TEXT,
            created_at DATETIME,
            updated_at DATETIME,
            FOREIGN KEY (role_id) REFERENCES roles(id)
        );
        CREATE TABLE IF NOT EXISTS role_permissions (
            role_id INTEGER,
            permission_id INTEGER,
            PRIMARY KEY(role_id, permission_id),
            FOREIGN KEY (role_id) REFERENCES roles(id),
            FOREIGN KEY (permission_id) REFERENCES permissions(id)
        );

        CREATE TABLE IF NOT EXISTS sections (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT UNIQUE
        );
        CREATE TABLE IF NOT EXISTS po_codes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT UNIQUE,
            created_at DATETIME
        );
        CREATE TABLE IF NOT EXISTS po_code_sections (
            po_code_id INTEGER,
            section_id INTEGER,
            PRIMARY KEY(po_code_id, section_id),
            FOREIGN KEY(po_code_id) REFERENCES po_codes(id) ON DELETE CASCADE,
            FOREIGN KEY(section_id) REFERENCES sections(id) ON DELETE CASCADE
        );
        CREATE TABLE IF NOT EXISTS process_order (
            process_order TEXT,
            posting_date TEXT,
            shift TEXT,
            description TEXT,
            line TEXT,
            po_code_id INTEGER,
            material_id INTEGER,
            FOREIGN KEY (po_code_id) REFERENCES po_codes(id)
            FOREIGN KEY (material_id) REFERENCES materials(id)
        );
        CREATE TABLE IF NOT EXISTS materials (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            code TEXT UNIQUE,
            key TEXT,
            value TEXT,
            created_at DATETIME
        );
        CREATE TABLE IF NOT EXISTS materials_value_description (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            value TEXT UNIQUE,
            desc TEXT
        );
        CREATE TABLE IF NOT EXISTS machines (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT UNIQUE,
            label TEXT,
            section_id INTEGER,
            FOREIGN KEY (section_id) REFERENCES sections(id)
        );
        CREATE TABLE IF NOT EXISTS user_sections (
            user_id INTEGER,
            section_id INTEGER,
            PRIMARY KEY(user_id, section_id),
            FOREIGN KEY(user_id) REFERENCES users(id),
            FOREIGN KEY(section_id) REFERENCES sections(id)
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
            name TEXT UNIQUE,
            section_id INTEGER,
            FOREIGN KEY (section_id) REFERENCES sections(id)
        );

        CREATE TABLE IF NOT EXISTS jobs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            machine_id INTEGER,
            shift_id INTEGER,
            created_by INTEGER,
            production_order TEXT,
            start_datetime DATETIME DEFAULT CURRENT_TIMESTAMP,
            end_datetime DATETIME,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (machine_id) REFERENCES machines(id),
            FOREIGN KEY (shift_id) REFERENCES shifts(id),
            FOREIGN KEY (created_by) REFERENCES users(id)
        );

        CREATE TABLE IF NOT EXISTS input_rolls (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            job_id INTEGER,
            batch TEXT,
            material_document TEXT DEFAULT NULL,
            material_number TEXT,
            start_meter DECIMAL(10,2),
            created_by INTEGER,
            start_weight DECIMAL(10,2),
            is_consumed INTEGER DEFAULT 0,
            consumed_at DATETIME DEFAULT NULL,
            consumed_weight DECIMAL(10,2) DEFAULT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (job_id) REFERENCES jobs(id),
            FOREIGN KEY (created_by) REFERENCES users(id)
        );

        CREATE TABLE IF NOT EXISTS output_rolls (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            input_roll_id INTEGER,
            from_input_batch TEXT,
            output_batch TEXT,
            final_meter DECIMAL(10,2),
            flag_reason TEXT,
            final_weight DECIMAL(10,2),
            core_weight DECIMAL(10,2),
            flag_count INTEGER DEFAULT 0,
            created_by INTEGER,
            updated_by INTEGER,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (input_roll_id) REFERENCES input_rolls(id),
            FOREIGN KEY (created_by) REFERENCES users(id),
            FOREIGN KEY (updated_by) REFERENCES users(id)
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
        ",
    )?;

    // Drop legacy rolls table and remove its permissions/content type
    conn.execute("DROP TABLE IF EXISTS rolls", [])?;
    conn.execute(
        "DELETE FROM role_permissions WHERE permission_id IN (
            SELECT p.id FROM permissions p
            JOIN content_type c ON p.content_type_id = c.id
            WHERE c.model = 'rolls'
        )",
        [],
    )?;
    conn.execute(
        "DELETE FROM permissions WHERE content_type_id IN (
            SELECT id FROM content_type WHERE model = 'rolls'
        )",
        [],
    )?;
    conn.execute("DELETE FROM content_type WHERE model = 'rolls'", [])?;

    // Initialize content types and permissions if they don't exist
    let models = vec![
        "users",
        "roles",
        "permissions",
        "jobs",
        "input_rolls",
        "output_rolls",
        "downtimes",
        "scraps",
        "ink_usages",
        "solvent_usages",
        "shifts",
        "colours",
        "solvent_types",
        "scrap_types",
        "downtime_reasons",
        "flag_reasons",
        "machines",
        "sections",
        "materials",
        "po_codes",
    ];

    for model in &models {
        // Insert content type if not exists
        conn.execute(
            "INSERT OR IGNORE INTO content_type (model) VALUES (?1)",
            rusqlite::params![model],
        )?;

        // Get content_type_id
        let content_type_id: i32 = conn.query_row(
            "SELECT id FROM content_type WHERE model = ?1",
            rusqlite::params![model],
            |row| row.get(0),
        )?;

        // Insert permissions if not exist
        let codename_create = format!("can_create_{}", model);
        let codename_read = format!("can_read_{}", model);
        let codename_update = format!("can_update_{}", model);
        let codename_delete = format!("can_delete_{}", model);

        let name_create = format!("Can create {}", model);
        let name_read = format!("Can read {}", model);
        let name_update = format!("Can update {}", model);
        let name_delete = format!("Can delete {}", model);

        conn.execute(
            "INSERT OR IGNORE INTO permissions (codename, name, content_type_id, can_create) 
             VALUES (?1, ?2, ?3, 1)",
            rusqlite::params![&codename_create, &name_create, content_type_id],
        )?;

        conn.execute(
            "INSERT OR IGNORE INTO permissions (codename, name, content_type_id, can_read) 
             VALUES (?1, ?2, ?3, 1)",
            rusqlite::params![&codename_read, &name_read, content_type_id],
        )?;

        conn.execute(
            "INSERT OR IGNORE INTO permissions (codename, name, content_type_id, can_update) 
             VALUES (?1, ?2, ?3, 1)",
            rusqlite::params![&codename_update, &name_update, content_type_id],
        )?;

        conn.execute(
            "INSERT OR IGNORE INTO permissions (codename, name, content_type_id, can_delete) 
             VALUES (?1, ?2, ?3, 1)",
            rusqlite::params![&codename_delete, &name_delete, content_type_id],
        )?;
    }

    // Get the read permission ID for this model (can_read)
    // Assign read access to all roles by default for all models
    let mut stmt = conn.prepare("SELECT id FROM permissions WHERE codename = ?1")?;

    for model in &models {
        let codename_read = format!("can_read_{}", model);
        let perm_id: i32 = match stmt.query_row(rusqlite::params![&codename_read], |row| row.get(0))
        {
            Ok(id) => id,
            Err(_) => continue,
        };

        // Get all roles
        let mut roles_stmt = conn.prepare("SELECT id FROM roles")?;
        let role_ids: Vec<i32> = roles_stmt
            .query_map([], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;

        // Assign read permission to each role if not already assigned
        for role_id in role_ids {
            conn.execute(
                "INSERT OR IGNORE INTO role_permissions (role_id, permission_id) VALUES (?1, ?2)",
                rusqlite::params![role_id, perm_id],
            )?;
        }
    }

    // Grant full access on all models to role_id = 1 by default
    let mut all_perm_stmt = conn.prepare("SELECT id FROM permissions")?;
    let all_perm_ids: Vec<i32> = all_perm_stmt
        .query_map([], |row| row.get(0))?
        .collect::<Result<Vec<_>, _>>()?;

    for perm_id in all_perm_ids {
        conn.execute(
            "INSERT OR IGNORE INTO role_permissions (role_id, permission_id) VALUES (?1, ?2)",
            rusqlite::params![1, perm_id],
        )?;
    }

    Ok(())
}

pub fn connect_local_db(path: &str) -> Result<Pool<SqliteConnectionManager>> {
    let manager = SqliteConnectionManager::file(path);
    let pool = Pool::new(manager).unwrap();
    Ok(pool)
}
