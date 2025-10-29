use actix_web::web;
use r2d2_sqlite::SqliteConnectionManager;
use r2d2::Pool;
use crate::backend::{handlers::*, middlewares::*, templates::{download_app, home_page, logout, roles_page, settings_page, signin_page, upload_app, user_page, whois_data}};

pub fn init_routes(cfg: &mut web::ServiceConfig, conn_data: web::Data<Pool<SqliteConnectionManager>>) {

    // User routes
    cfg.service(
        web::scope("/api/users")
            .service(web::resource("").wrap(CheckRead { model: "users", conn_data: conn_data.clone() }).route(web::get().to(all_users)))
            .service(web::resource("/me").wrap(CheckRead { model: "users", conn_data: conn_data.clone() }).route(web::get().to(get_me)))
            .service(web::resource("/create").wrap(CheckCreate { model: "users", conn_data: conn_data.clone() }).route(web::post().to(create_user)))
            .service(web::resource("/update").wrap(CheckUpdate { model: "users", conn_data: conn_data.clone() }).route(web::put().to(update_user)))
            .service(web::resource("/delete").wrap(CheckDelete { model: "users", conn_data: conn_data.clone() }).route(web::delete().to(delete_user)))
    );

    // Permission routes
    cfg.service(
        web::scope("/api/permissions")
            .service(web::resource("").wrap(CheckRead { model: "users", conn_data: conn_data.clone() }).route(web::get().to(all_permissions)))
            .service(web::resource("/create").wrap(CheckCreate { model: "users", conn_data: conn_data.clone() }).route(web::post().to(create_permission)))
            .service(web::resource("/update").wrap(CheckUpdate { model: "users", conn_data: conn_data.clone() }).route(web::put().to(update_permission)))
            .service(web::resource("/delete").wrap(CheckDelete { model: "users", conn_data: conn_data.clone() }).route(web::delete().to(delete_permission)))
    );

    // Role routes
    cfg.service(
        web::scope("/api/roles")
            .service(web::resource("").wrap(CheckRead { model: "users", conn_data: conn_data.clone() }).route(web::get().to(all_roles)))
            .service(web::resource("/create").wrap(CheckCreate { model: "roles", conn_data: conn_data.clone() }).route(web::post().to(create_role)))
            .service(web::resource("/update").wrap(CheckUpdate { model: "roles", conn_data: conn_data.clone() }).route(web::put().to(update_role)))
            .service(web::resource("/delete").wrap(CheckDelete { model: "roles", conn_data: conn_data.clone() }).route(web::delete().to(delete_role)))
    );

    // Auth routes
    cfg.service(
        web::scope("/auth")
            .route("/signin", web::post().to(signin_user))
            .route("/signin", web::get().to(signin_page))
            .route("/signout", web::get().to(signout_user))
    );

    // Others routes
    cfg.service(
        web::scope("")
            .service(web::resource("/").wrap(CheckRead { model: "users", conn_data: conn_data.clone() }).route(web::get().to(home_page)))
            .service(web::resource("/upload/{name}").route(web::post().to(upload_app)))
            .service(web::resource("/download/{name}").route(web::get().to(download_app)))
            .service(web::resource("/users").wrap(CheckRead { model: "users", conn_data: conn_data.clone() }).route(web::get().to(user_page)))
            .service(web::resource("/roles").wrap(CheckRead { model: "roles", conn_data: conn_data.clone() }).route(web::get().to(roles_page)))
            .service(web::resource("/settings").wrap(CheckRead { model: "users", conn_data: conn_data.clone() }).route(web::get().to(settings_page)))
            .route("/whois", web::get().to(whois_data))
            .route("/logout", web::get().to(logout))
            .route("/{_:.*}", web::get().to(signin_page))
    );
}
