use actix_cors::Cors;
use anyhow::Context;
use actix_files::Files;
use std::{io, fs, process};
use actix_web::{web, App, HttpServer};
use insignia_mes::hardware::routes::init_routes;


#[actix_web::main]
async fn main() -> io::Result<()> {
    let pid_file = "data/hardware.pid";
    if let Ok(existing) = fs::read_to_string(pid_file) {
        if let Ok(pid) = existing.trim().parse::<i32>() {
            let _ = process::Command::new("taskkill")
                .args(&["/PID", &pid.to_string(), "/F"])
                .status();
        }
    }
    let _ = fs::write(pid_file, process::id().to_string()).context("Failed to write PID file");
    let server = HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .service(Files::new("/static", "./static"))
            .service(
                web::scope("").configure(|cfg| init_routes(cfg))
            )
    })
    .bind(("localhost", 8080))?;
    server.run().await
}


