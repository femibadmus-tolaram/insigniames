use std::env;
use std::error;

use tiberius::{Client, Config, AuthMethod};
use tokio::net::TcpStream;
use tokio_util::compat::TokioAsyncWriteCompatExt;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;



pub async fn sync_scrap_data(local_pool: &Pool<SqliteConnectionManager>) -> Result<(), Box<dyn error::Error>> {
    let server = env::var("SCRAP_SERVER")?;
    let database = env::var("SCRAP_DB")?;
    let username = env::var("SCRAP_UID")?;
    let password = env::var("SCRAP_PASSWD")?;

    let mut config = Config::new();
    config.host(&server);
    config.port(1433);
    config.database(&database);
    config.authentication(AuthMethod::sql_server(&username, &password));
    config.trust_cert();
    
    let conn = local_pool.get()?;
    
    let last_sync_time: Option<String> = conn.query_row(
        "SELECT MAX(time) FROM scraps",
        [],
        |row| row.get(0)
    ).ok();
    
    let query = if let Some(last_time) = last_sync_time {
        format!(
            "SELECT 
                CAST(REPLACE(w.shift, 'SHIFT ', '') AS int) AS [Shift],
                w.actualdate + ' ' + w.log_time AS [DateTime],
                w.materials AS [ScrapType],
                w.weight AS [Weight],
                CONCAT(u.firstname, ' ', u.surname) AS [CreatedBy]
            FROM weight_log w
            LEFT JOIN Users u ON w.staffid = u.staffid
            WHERE w.actualdate + ' ' + w.log_time > '{}'
            ORDER BY w.actualdate, w.log_time",
            last_time
        )
    } else {
        "SELECT 
            CAST(REPLACE(w.shift, 'SHIFT ', '') AS int) AS [Shift],
            w.actualdate + ' ' + w.log_time AS [DateTime],
            w.materials AS [ScrapType],
            w.weight AS [Weight],
            CONCAT(u.firstname, ' ', u.surname) AS [CreatedBy]
        FROM weight_log w
        LEFT JOIN Users u ON w.staffid = u.staffid
        ORDER BY w.actualdate, w.log_time".to_string()
    };

    let tcp = TcpStream::connect(config.get_addr()).await?;
    tcp.set_nodelay(true)?;
    
    let mut client = Client::connect(config, tcp.compat_write()).await?;
    let stream = client.query(&query, &[]).await?;
    let rows = stream.into_results().await?;

    for row in rows {
        for col in row {
            let shift_num: Option<i32> = col.get("Shift");
            let datetime_str: Option<&str> = col.get("DateTime");
            let scrap_type_name: Option<&str> = col.get("ScrapType");
            let weight: Option<f64> = col.get("Weight");
            let created_by_name: Option<&str> = col.get("CreatedBy");

            if let (Some(shift_num), Some(datetime_str), Some(scrap_type_name), Some(weight), Some(created_by_name)) = 
                (shift_num, datetime_str, scrap_type_name, weight, created_by_name) {
                
                let datetime = parse_datetime(datetime_str)?;
                
                let shift_name = match shift_num {
                    1 => "SHIFT 1",
                    2 => "SHIFT 2",
                    _ => "SHIFT 1",
                };
                
                let shift_id = get_or_create_shift(&conn, shift_name)?;
                let scrap_type_id = get_or_create_scrap_type(&conn, scrap_type_name)?;
                let user_id = get_or_create_user(&conn, created_by_name)?;

                conn.execute(
                    "INSERT INTO scraps (shift_id, time, scrap_type_id, weight_kg, created_by, created_at, updated_at)
                     VALUES (?, ?, ?, ?, ?, datetime('now'), datetime('now'))",
                    params![shift_id, datetime, scrap_type_id, weight, user_id],
                )?;
            }
        }
    }
    
    Ok(())
}

fn parse_datetime(datetime_str: &str) -> Result<String, Box<dyn error::Error>> {
    let parts: Vec<&str> = datetime_str.split_whitespace().collect();
    if parts.len() != 2 {
        return Ok(datetime_str.to_string());
    }
    
    let date_part = parts[0];
    let time_part = parts[1];
    
    let date_parts: Vec<&str> = date_part.split('-').collect();
    if date_parts.len() != 3 {
        return Ok(datetime_str.to_string());
    }
    
    let day = format!("{:02}", date_parts[0].parse::<u32>().unwrap_or(0));
    let month_str = date_parts[1].to_uppercase();
    let year = date_parts[2];
    
    let month_num = match month_str.as_str() {
        "JAN" => "01",
        "FEB" => "02",
        "MAR" => "03",
        "APR" => "04",
        "MAY" => "05",
        "JUN" => "06",
        "JUL" => "07",
        "AUG" => "08",
        "SEP" => "09",
        "OCT" => "10",
        "NOV" => "11",
        "DEC" => "12",
        _ => return Ok(datetime_str.to_string()),
    };
    
    let time_clean = time_part.to_lowercase();
    let is_pm = time_clean.contains("pm");
    
    let time_without_ampm = time_clean
        .replace("am", "")
        .replace("pm", "");
    
    let time_parts: Vec<&str> = time_without_ampm.split(':').collect();
    if time_parts.len() != 3 {
        return Ok(datetime_str.to_string());
    }
    
    let mut hour: u32 = time_parts[0].parse().unwrap_or(0);
    let minute = time_parts[1];
    let second = time_parts[2];
    
    if is_pm && hour < 12 {
        hour += 12;
    } else if !is_pm && hour == 12 {
        hour = 0;
    }
    
    let hour_str = format!("{:02}", hour);
    
    Ok(format!("{}-{}-{} {}:{}:{}", year, month_num, day, hour_str, minute, second))
}

fn get_or_create_shift(conn: &rusqlite::Connection, name: &str) -> rusqlite::Result<i64> {
    let shift_id: Result<i64, _> = conn.query_row(
        "SELECT id FROM shifts WHERE name = ?",
        params![name],
        |row| row.get(0),
    );
    
    match shift_id {
        Ok(id) => Ok(id),
        Err(_) => {
            conn.execute("INSERT INTO shifts (name) VALUES (?)", params![name])?;
            Ok(conn.last_insert_rowid())
        }
    }
}

fn get_or_create_scrap_type(conn: &rusqlite::Connection, name: &str) -> rusqlite::Result<i64> {
    let scrap_type_id: Result<i64, _> = conn.query_row(
        "SELECT id FROM scrap_types WHERE name = ?",
        params![name],
        |row| row.get(0),
    );
    
    match scrap_type_id {
        Ok(id) => Ok(id),
        Err(_) => {
            conn.execute("INSERT INTO scrap_types (name) VALUES (?)", params![name])?;
            Ok(conn.last_insert_rowid())
        }
    }
}

fn get_or_create_user(conn: &rusqlite::Connection, full_name: &str) -> rusqlite::Result<i64> {
    let user_id: Result<i64, _> = conn.query_row(
        "SELECT id FROM users WHERE full_name = ?",
        params![full_name],
        |row| row.get(0),
    );
    
    match user_id {
        Ok(id) => Ok(id),
        Err(_) => {
            let staffid = format!("EXT-{}", chrono::Local::now().timestamp());
            conn.execute(
                "INSERT INTO users (full_name, staffid, status, role_id, created_at, updated_at) 
                 VALUES (?, ?, 'active', 1, datetime('now'), datetime('now'))",
                params![full_name, staffid],
            )?;
            Ok(conn.last_insert_rowid())
        }
    }
}


