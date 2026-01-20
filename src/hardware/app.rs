use crate::manager::config::*;
use actix_web::{HttpResponse, Responder, web};
use base64::prelude::*;
use chrono::Local;
use reqwest;
use serde::Deserialize;
use serde_json::json;
use std::{ffi::CString, fs, path::Path, ptr};
#[cfg(windows)]
use winapi::um::processthreadsapi::{CreateProcessA, PROCESS_INFORMATION, STARTUPINFOA};

pub async fn run_update(
    base_url: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    let result = (|| -> Result<String, Box<dyn std::error::Error>> {
        let base_url = base_url
            .get("base_url")
            .ok_or("Missing base_url parameter")?;
        if !Path::new("update.exe").exists() {
            download_update(base_url)?;
        }
        download_app(base_url)?;
        fs::write(
            "data/last_update.txt",
            Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        )?;
        #[cfg(windows)]
        unsafe {
            let cmd = CString::new("update.exe").unwrap();
            let mut startup_info: STARTUPINFOA = std::mem::zeroed();
            let mut process_info: PROCESS_INFORMATION = std::mem::zeroed();
            startup_info.cb = std::mem::size_of::<STARTUPINFOA>() as u32;
            CreateProcessA(
                ptr::null(),
                cmd.as_ptr() as *mut i8,
                ptr::null_mut(),
                ptr::null_mut(),
                0,
                0,
                ptr::null_mut(),
                ptr::null(),
                &mut startup_info,
                &mut process_info,
            );
        }
        #[cfg(not(windows))]
        {
            // On non-Windows, just skip process creation or implement alternative if needed
        }
        Ok("Update started - application will restart shortly...".to_string())
    })();

    match result {
        Ok(msg) => HttpResponse::Ok().body(msg),
        Err(e) => HttpResponse::InternalServerError().body(format!("Update failed: {}", e)),
    }
}

pub async fn get_last_update() -> impl Responder {
    match fs::read_to_string("data/last_update.txt") {
        Ok(content) => HttpResponse::Ok().body(content),
        Err(_) => HttpResponse::Ok().body("No update record found"),
    }
}

pub fn download_app(base_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!("{}/download/service", base_url);
    let mut response = reqwest::blocking::get(url)?;
    if !response.status().is_success() {
        return Err(format!("Failed to download app: {}", response.status()).into());
    }
    let mut file = fs::File::create("new_app.exe")?;
    response.copy_to(&mut file)?;
    Ok(())
}

pub fn download_update(base_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!("{}/download/update", base_url);
    let mut response = reqwest::blocking::get(url)?;
    if !response.status().is_success() {
        return Err(format!("Failed to download updater: {}", response.status()).into());
    }
    let mut file = fs::File::create("update.exe")?;
    response.copy_to(&mut file)?;
    Ok(())
}

#[derive(Deserialize)]
pub struct PrintData {
    pub pdf_data: String,
}

pub async fn scan_qrcode() -> impl Responder {
    let scanner_config = AppConfig::scanner();

    if scanner_config.port_name.is_empty() {
        return HttpResponse::BadRequest().json(json!({
            "success": false,
            "message": "Scanner configuration not found",
            "data": null
        }));
    }

    match serialport::new(&scanner_config.port_name, scanner_config.baud_rate)
        .timeout(std::time::Duration::from_millis(1000))
        .open()
    {
        Ok(mut port) => {
            let mut buf = [0u8; 1024];
            let mut temp = Vec::new();

            loop {
                match port.read(&mut buf) {
                    Ok(n) if n > 0 => {
                        temp.extend_from_slice(&buf[..n]);
                        while let Some(pos) = temp.iter().position(|&b| b == b'\n' || b == b'\r') {
                            let line_bytes = temp.drain(..=pos).collect::<Vec<u8>>();
                            let line = String::from_utf8_lossy(&line_bytes)
                                .trim_end_matches(&['\r', '\n'][..])
                                .to_string();

                            if !line.is_empty() {
                                return HttpResponse::Ok().json(json!({
                                    "success": true,
                                    "message": "QR code scanned successfully",
                                    "data": line
                                }));
                            }
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => continue,
                    Err(_) => {
                        return HttpResponse::BadRequest().json(json!({
                            "success": false,
                            "message": "Scanner disconnected",
                            "data": null
                        }));
                    }
                    Ok(_) => continue,
                }
            }
        }
        Err(_) => {
            return HttpResponse::BadRequest().json(json!({
                "success": false,
                "message": "Scanner disconnected",
                "data": null
            }));
        }
    }
}

pub async fn get_settings() -> HttpResponse {
    let config = serde_json::json!({
        "scanner": AppConfig::scanner(),
        "scale": AppConfig::scale(),
    });
    HttpResponse::Ok().json(config)
}

pub async fn update_settings(new_config: web::Json<serde_json::Value>) -> HttpResponse {
    let scanner =
        serde_json::from_value::<DeviceConfig>(new_config["scanner"].clone()).unwrap_or_default();
    let scale =
        serde_json::from_value::<DeviceConfig>(new_config["scale"].clone()).unwrap_or_default();
    AppConfig::save_to_env(scanner.clone(), scale.clone());
    HttpResponse::Ok().json(serde_json::json!({ "scanner": scanner, "scale": scale }))
}

pub async fn test_scanner_connection() -> HttpResponse {
    match AppConfig::test_scanner_connection() {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": "Scanner connection successful"
        })),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
            "success": false,
            "message": e
        })),
    }
}

pub async fn test_scale_connection() -> HttpResponse {
    let scale = AppConfig::get_scale();
    match scale.test_connection() {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": "Scale connection successful"
        })),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
            "success": false,
            "message": e
        })),
    }
}

pub async fn test_printer_connection() -> HttpResponse {
    if check_printer() {
        return HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": "Printer connection successful"
        }));
    } else {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "success": false,
            "message": "Printer not found"
        }));
    }
}

pub async fn print_qrcode(data: web::Json<PrintData>) -> HttpResponse {
    let pdf_data = &data.pdf_data;
    let clean_data = if pdf_data.starts_with("data:") {
        pdf_data.split("base64,").last().unwrap_or(pdf_data)
    } else {
        pdf_data
    };
    match BASE64_STANDARD.decode(clean_data) {
        Ok(decoded_data) => {
            match print_pdf(&decoded_data) {
                Ok(_) => HttpResponse::Ok().json(serde_json::json!({
                    "success": true,
                    "message": "Print successfully"
                })),
                // Err(_) => HttpResponse::Ok().json(serde_json::json!({
                //     "success": true,
                //     "message": "Print successfully"
                // })),
                Err(e) => HttpResponse::InternalServerError().body(format!("Print failed: {}", e)),
            }
        }
        Err(e) => HttpResponse::BadRequest().body(format!("Invalid base64: {}", e)),
    }
}

pub async fn get_scale_weight() -> HttpResponse {
    let scale_config = AppConfig::scale();
    let scale = Scale::new(scale_config.port_name, scale_config.baud_rate);
    match scale.get_weight() {
        Ok(weight) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "weight": weight
        })),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
            "success": false,
            "message": e
        })),
        // Err(_) => HttpResponse::Ok().json(serde_json::json!({
        //     "success": true,
        //     "weight": 8.5
        // })),
    }
}
