use actix_web::web;
use dotenvy::dotenv;
use insignia_mes::backend::app::start_backend;
use insignia_mes::manager::db::{connect_local_db, init_local_db};
use insignia_mes::sap::{sync_material_codes, sync_process_orders, sync_scrap_data};
use std::{fs, process};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let db_file = "data/local.db";
    let pid_file = "data/backend.pid";
    fs::create_dir_all("data").expect("Failed to create backups directory");

    if let Ok(existing) = fs::read_to_string(pid_file) {
        if let Ok(pid) = existing.trim().parse::<i32>() {
            let _ = process::Command::new("taskkill")
                .args(&["/PID", &pid.to_string(), "/F"])
                .status();
        }
    }

    fs::write(pid_file, process::id().to_string())?;
    init_local_db(db_file).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    let local_pool =
        connect_local_db(db_file).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    let local_pool_clone = local_pool.clone();
    let local_pool_clone2 = local_pool.clone();
    tokio::spawn(async move {
        loop {
            if let Err(e) = sync_scrap_data(&local_pool_clone).await {
                eprintln!("Warning: Failed to sync scrap data: {}", e);
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;
        }
    });
    tokio::spawn(async move {
        loop {
            if let Err(e) = sync_process_orders(&local_pool_clone2).await {
                eprintln!("Warning: Failed to sync process order: {}", e);
            }
            if let Err(e) = sync_material_codes(&local_pool_clone2).await {
                eprintln!("Warning: Failed to sync materials: {}", e);
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(600)).await;
        }
    });

    let local_pool_data = web::Data::new(local_pool);
    start_backend(local_pool_data, 911)
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}
