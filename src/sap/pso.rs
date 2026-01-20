use chrono::Datelike;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use reqwest;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::{env, error};

#[derive(Debug, Deserialize)]
struct ApiResponse {
    d: ResponseData,
}

#[derive(Debug, Deserialize)]
struct ResponseData {
    results: Vec<ProcessOrder>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ProcessOrder {
    #[serde(rename = "ProcessOrder")]
    process_order: String,
    #[serde(rename = "PostingDate")]
    posting_date: String,
    #[serde(rename = "Shift")]
    shift: String,
    #[serde(rename = "Material")]
    material: String,
    #[serde(rename = "MaterialDescription")]
    material_description: String,
    #[serde(rename = "Line")]
    line: String,
}

pub async fn sync_process_orders(
    local_pool: &Pool<SqliteConnectionManager>,
) -> Result<(), Box<dyn error::Error>> {
    let api_key = env::var("SAP_QA_PSO_APIKey").unwrap_or_default();
    let base_url = env::var("SAP_QA_PSO_BASE_URL").unwrap_or_default();

    let conn = local_pool.get()?;

    let _starget_date = get_target_date(&conn)?;

    for day_offset in (0..=5).rev() {
        let today = chrono::Local::now().date_naive();
        let target_date = today - chrono::Days::new(day_offset);

        let formatted_date = format!(
            "{}-{:02}-{:02}",
            target_date.year(),
            target_date.month(),
            target_date.day()
        );
        let url = format!(
            "{}?$format=json&$filter=OrderType eq 'ZIS1' and Plant eq 'A710' and PostingDate eq datetime'{}T00:00:00'",
            base_url, formatted_date
        );
        // print!("{}", url);

        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .header("APIKey", api_key.clone())
            .send()
            .await?;
        let status = response.status();
        let text = response.text().await?;

        if status.is_success() {
            let api_response: ApiResponse = serde_json::from_str(&text)?;
            for po in api_response.d.results {
                extract_and_save_po_data(&conn, &po)?;
            }
        }
    }
    Ok(())
}

fn get_target_date(conn: &rusqlite::Connection) -> rusqlite::Result<chrono::NaiveDate> {
    let today = chrono::Local::now().date_naive();

    let last_date: Option<String> =
        conn.query_row("SELECT MAX(posting_date) FROM process_order", [], |row| {
            row.get(0)
        })?;

    let target_date = last_date
        .and_then(|date_str| chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").ok())
        .and_then(|date| date.succ_opt())
        .unwrap_or(today);

    Ok(target_date)
}

fn extract_and_save_po_data(
    conn: &rusqlite::Connection,
    po: &ProcessOrder,
) -> rusqlite::Result<()> {
    let (po_code, rest_description) = extract_po_and_rest(&po.material_description);

    let po_code_id = get_or_create_po_code(conn, &po_code)?;
    let material_id = get_or_create_material(conn, &po.process_order)?;

    let posting_date = parse_sap_date(&po.posting_date)?;

    conn.execute(
        "INSERT INTO process_order (
            process_order, posting_date, shift, description, 
            line, po_code_id, material_id
        ) SELECT ?, ?, ?, ?, ?, ?, ?
        WHERE NOT EXISTS (
            SELECT 1 FROM process_order WHERE process_order = ?
        )",
        params![
            po.process_order,
            posting_date,
            po.shift,
            rest_description,
            po.line,
            po_code_id,
            material_id,
            po.process_order,
        ],
    )?;

    Ok(())
}

fn get_or_create_material(conn: &rusqlite::Connection, code: &str) -> rusqlite::Result<i64> {
    let existing_id: Option<i64> = conn
        .query_row(
            "SELECT id FROM materials WHERE code = ?",
            params![code],
            |row| row.get(0),
        )
        .ok();

    if let Some(id) = existing_id {
        return Ok(id);
    }

    conn.execute(
        "INSERT INTO materials (code, key, value, created_at) VALUES (?, 'Loading...', 'Loading...', datetime('now'))",
        params![code],
    )?;

    Ok(conn.last_insert_rowid())
}

fn extract_po_and_rest(material_desc: &str) -> (String, String) {
    if let Some(pos) = material_desc.find('-') {
        let po_code = material_desc[..pos].trim().to_string();
        let rest = material_desc[pos + 1..].trim().to_string();
        (po_code, rest)
    } else {
        let first_word = material_desc
            .split_whitespace()
            .next()
            .unwrap_or("")
            .to_string();
        (first_word.clone(), first_word)
    }
}

fn get_or_create_po_code(conn: &rusqlite::Connection, po_code: &str) -> rusqlite::Result<i64> {
    if po_code.is_empty() {
        return Ok(0);
    }
    let existing_id: Option<i64> = conn
        .query_row(
            "SELECT id FROM po_codes WHERE name = ?",
            params![po_code],
            |row| row.get(0),
        )
        .ok();

    if let Some(id) = existing_id {
        return Ok(id);
    }
    conn.execute(
        "INSERT INTO po_codes (name, created_at) VALUES (?, datetime('now'))",
        params![po_code],
    )?;

    let new_id = conn.last_insert_rowid();
    Ok(new_id)
}

fn parse_sap_date(sap_date: &str) -> rusqlite::Result<String> {
    if sap_date.starts_with("/Date(") && sap_date.ends_with(")/") {
        let timestamp_str = &sap_date[6..sap_date.len() - 2];
        if let Ok(timestamp_ms) = timestamp_str.parse::<i64>() {
            let timestamp_secs = timestamp_ms / 1000;
            let date_time = chrono::DateTime::from_timestamp(timestamp_secs, 0)
                .unwrap_or_else(|| chrono::DateTime::from_timestamp(0, 0).unwrap());
            return Ok(date_time.format("%Y-%m-%d").to_string());
        }
    }

    Ok(chrono::Local::now().format("%Y-%m-%d").to_string())
}
