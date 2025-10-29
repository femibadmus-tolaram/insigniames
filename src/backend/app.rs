use actix_session::{SessionMiddleware, storage::CookieSessionStore};
use actix_web::{cookie::Key, web, App, HttpServer};
use actix_files::Files;
// use tokio::sync::watch::Sender;
// use actix_session::config::PersistentSession;
use r2d2_sqlite::SqliteConnectionManager;
use r2d2::Pool;
use actix_cors::Cors;


use crate::backend::routes;

pub async fn start_backend(local_pool: web::Data<Pool<SqliteConnectionManager>>, port: u16) -> std::io::Result<()> {
    let secret_key = Key::from(&[0; 64]);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .app_data(local_pool.clone())
            .wrap(SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
                .cookie_secure(false)
                // .session_lifecycle(PersistentSession::default().session_ttl(actix_web::cookie::time::Duration::hours(12)))
                .build()
            )
            .service(Files::new("/static", "./static"))
            .service(
                web::scope("").configure(|cfg| routes::init_routes(cfg, local_pool.clone()))
            )
    })
    .bind(("0.0.0.0", port.clone()))?;
    server.run().await
}
