use serde_json::json;
use reqwest;
use r2d2::Pool;
use reqwest::Client;
use rusqlite::params;
use std::{env, error, collections};
use r2d2_sqlite::SqliteConnectionManager;

pub async fn get_material_descriptions(matnrs: Vec<String>) -> Result<collections::HashMap<String, String>, Box<dyn error::Error>> {
    let client = Client::new();
    
    let token_url = format!("https://login.microsoftonline.com/{}/oauth2/v2.0/token", env::var("FABRIC_TENANT_ID")?);
    let token_res = client.post(&token_url)
        .form(&[
            ("client_id", env::var("FABRIC_CLIENT_ID")?),
            ("client_secret", env::var("FABRIC_CLIENT_SECRET")?),
            ("scope", "https://api.fabric.microsoft.com/.default".to_owned()),
            ("grant_type", "client_credentials".to_owned())
        ])
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;
    
    let token = token_res["access_token"].as_str().unwrap();

    let first_value = if matnrs.len() > 0 { matnrs.len() } else { 50 };

    let formatted_matnrs: Vec<String> = matnrs.into_iter()
        .map(|matnr| format!("{:0>18}", matnr))
        .collect();
    
    let query = json!({
        "query": format!("
            query ($matnrs: [String!]!) {{
            mAKTs(first: {}, filter: {{ MATNR: {{ in: $matnrs }} }}) {{
                items {{ MATNR MAKTX }}
            }}
            }}
        ", first_value),
        "variables": {
            "matnrs": formatted_matnrs
        }
    });

    let url = env::var("FABRIC_AUTH_API")?;
    
    let res = client.post(&url)
        .json(&query)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;
    
    let mut descriptions = std::collections::HashMap::new();
    
    if let Some(items) = res["data"]["mAKTs"]["items"].as_array() {
        for item in items {
            if let (Some(matnr), Some(maktx)) = (item["MATNR"].as_str(), item["MAKTX"].as_str()) {
                let trimmed_matnr = matnr.trim_start_matches('0');
                descriptions.insert(trimmed_matnr.to_string(), maktx.to_string());
            }
        }
    }
    
    Ok(descriptions)
}

pub async fn sync_material_codes(local_pool: &Pool<SqliteConnectionManager>) -> Result<(), Box<dyn error::Error>> {
    let user = env::var("SAP_QA_PDO_USERNAME").unwrap_or_default();
    let pass = env::var("SAP_QA_PDO_PASSWORD").unwrap_or_default();
    let base_url = env::var("SAP_QA_PDO_URL").unwrap_or_default();

    let conn = local_pool.get()?;
    let codes = get_materials_needing_update(&conn)?;

    for code in codes {
        let url = format!("{}?$format=json&$filter=Plant eq 'A710' and ManufacturingOrder eq '{}'", base_url, code);
        let url = url + "&$expand=to_ProductionOrderComponent,to_ProductionOrderOperation";

        let client = Client::new();
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("sap-client", reqwest::header::HeaderValue::from_static("500"));

        let res = client
            .get(&url)
            .headers(headers)
            .basic_auth(&user, Some(&pass))
            .send()
            .await?;

        let text = res.text().await?;
        if text.is_empty() {
            continue;
        }

        let json: serde_json::Value = serde_json::from_str(&text)?;
        let results = json["d"]["results"].as_array().ok_or("No results array")?;
        let first_result = results.first().ok_or(format!("No first result: {}", text))?;
        let components = first_result["to_ProductionOrderComponent"]["results"].as_array().ok_or("No components array")?;

        let mut keys = Vec::new();
        let mut value_ids = Vec::new();

        for component in components {
            let material_group = component["MaterialGroup"].as_str().ok_or("No MaterialGroup")?;
            let material = component["Material"].as_str().ok_or("No Material")?;
            
            keys.push(material_group.to_string());
            let value_id = get_or_create_value(&conn, material)?;
            value_ids.push(value_id);
        }

        let keys_str = keys.join(",");
        let value_ids_str: Vec<String> = value_ids.iter().map(|id| id.to_string()).collect();
        let value_ids_joined = value_ids_str.join(",");
        
        update_material(&conn, &code, &keys_str, &value_ids_joined)?;
    }

    update_value_descriptions(local_pool).await?;
    
    Ok(())
}

pub async fn update_value_descriptions(local_pool: &Pool<SqliteConnectionManager>) -> Result<(), Box<dyn error::Error>> {
    let conn = local_pool.get()?;
    
    let (numeric_values, id_value_map) = {
        let mut stmt = conn.prepare("SELECT id, value FROM materials_value_description WHERE desc IS NULL OR desc = ''")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })?;
        
        let mut numeric_values = Vec::new();
        let mut id_value_map = std::collections::HashMap::new();
        
        for row in rows {
            let (id, value) = row?;
            numeric_values.push(value.clone());
            id_value_map.insert(id, value);
        }
        
        (numeric_values, id_value_map)
    }; // stmt dropped here
    
    let descriptions = get_material_descriptions(numeric_values).await?;
    
    for (id, value) in id_value_map {
        if let Some(description) = descriptions.get(&value) {
            conn.execute(
                "UPDATE materials_value_description SET desc = ? WHERE id = ?",
                params![description, id],
            )?;
        }
    }
    
    Ok(())
}

fn get_materials_needing_update(conn: &rusqlite::Connection) -> rusqlite::Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT code FROM materials WHERE (key IS NULL OR key = '' OR key = 'Loading...') OR (value IS NULL OR value = '' OR value = 'Loading...')")?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
    rows.collect()
}

fn update_material(conn: &rusqlite::Connection, code: &str, key: &str, value_ids: &str) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE materials SET key = ?, value = ? WHERE code = ?",
        params![key, value_ids, code],
    )?;
    Ok(())
}

fn get_or_create_value(conn: &rusqlite::Connection, value: &str) -> rusqlite::Result<i64> {
    let existing_id: Result<i64, _> = conn.query_row(
        "SELECT id FROM materials_value_description WHERE value = ?",
        params![value],
        |row| row.get(0),
    );

    match existing_id {
        Ok(id) => Ok(id),
        Err(_) => {
            conn.execute(
                "INSERT INTO materials_value_description (value) VALUES (?)",
                params![value],
            )?;
            Ok(conn.last_insert_rowid())
        }
    }
}

