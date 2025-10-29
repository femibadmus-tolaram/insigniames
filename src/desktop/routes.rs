use actix_web::web;
use crate::desktop::app::*;

pub fn init_routes(cfg: &mut web::ServiceConfig) {

    // Settings
    cfg.service(
        web::scope("/api/settings")
            .service(web::resource("").route(web::get().to(get_settings)))
            .service(web::resource("/update").route(web::put().to(update_settings)))
            .service(web::resource("/test/scale").route(web::get().to(test_scale_connection)))
            .service(web::resource("/test/scanner").route(web::get().to(test_scanner_connection)))
            .service(web::resource("/test/printer").route(web::get().to(test_printer_connection)))
    );

    // App
    cfg.service(
        web::scope("/api/app")
            .route("/scan", web::post().to(scan_qrcode))
            .route("/print", web::post().to(print_qrcode))
            .route("/weight", web::get().to(get_scale_weight))
            .route("/last_update", web::get().to(get_last_update))
            .route("/update", web::get().to(run_update))
    );
}
