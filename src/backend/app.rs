use actix_cors::Cors;
use actix_files::Files;
use actix_session::{SessionMiddleware, storage::CookieSessionStore};
use actix_web::{App, HttpResponse, HttpServer, Responder, cookie::Key, web, middleware::Logger};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rust_embed::RustEmbed;

use crate::backend::routes;

#[derive(RustEmbed)]
#[folder = "static/"]
struct Asset;

async fn static_handler(path: web::Path<String>) -> impl Responder {
    let file_path = path.into_inner();
    if let Some(file) = Asset::get(&file_path) {
        let mime = mime_guess::from_path(&file_path).first_or_octet_stream();
        HttpResponse::Ok()
            .content_type(mime.as_ref())
            .body(file.data.into_owned())
    } else {
        HttpResponse::NotFound().body("not found")
    }
}

pub async fn start_backend(
    local_pool: web::Data<Pool<SqliteConnectionManager>>,
    port: u16,
) -> std::io::Result<()> {
    let secret_key = Key::from(&[0; 64]);
    HttpServer::new(move || {
        App::new()
            .wrap(
                Logger::new("%a \"%r\" %s %b \"%{Referer}i\" \"%{User-Agent}i\" %T")
                    .log_target("access"),
            )
            .wrap(Cors::permissive())
            .app_data(local_pool.clone())
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
                    .cookie_secure(false)
                    .build(),
            )
            .route("/static/{_:.*}", web::get().to(static_handler))
            .service(Files::new("/static_fs", "./static"))
            .service(web::scope("").configure(|cfg| routes::init_routes(cfg, local_pool.clone())))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
