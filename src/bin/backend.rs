use actix_web::web;
use insignia_mes::manager::db::{connect_local_db, init_local_db};
use insignia_mes::backend::app::start_backend;
use std::{fs, process};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db_file = "data/local.db";
    let pid_file = "data/app.pid";
    let backup_path = "data/backups";
    fs::create_dir_all(backup_path).expect("Failed to create backups directory");

    if let Ok(existing) = fs::read_to_string(pid_file) {
        if let Ok(pid) = existing.trim().parse::<i32>() {
            let _ = process::Command::new("taskkill")
                .args(&["/PID", &pid.to_string(), "/F"])
                .status();
        }
    }

    fs::write(pid_file, process::id().to_string())?;
    init_local_db(db_file).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    let local_pool = connect_local_db(db_file).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    let local_pool_data = web::Data::new(local_pool);
    start_backend(local_pool_data, 80)
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}
