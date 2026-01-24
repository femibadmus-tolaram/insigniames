use actix_web::{web, HttpResponse, Responder};
use r2d2_sqlite::SqliteConnectionManager;
use r2d2::Pool;
use crate::backend::models::{Scrap, ScrapCreatePayload, ScrapPayload, IdPayload, ScrapFilterPayload};

pub async fn create_scrap(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<ScrapCreatePayload>, session: actix_session::Session) -> impl Responder {
    let user_id: Option<i32> = session.get("user_id").unwrap_or(None);
    if user_id.is_none() {
        return HttpResponse::Unauthorized().body("User not authenticated");
    }
    
    let conn = conn_data.get().unwrap();
    match Scrap::create(&conn, &data, user_id.unwrap()) {
        Ok(scrap) => HttpResponse::Ok().json(scrap),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn update_scrap(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<ScrapPayload>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match Scrap::find_by_id(&conn, data.id) {
        Ok(mut scrap) => {
            if let Err(e) = scrap.update(&conn, &data) { return HttpResponse::InternalServerError().body(e.to_string()); }
            HttpResponse::Ok().json(scrap)
        }
        Err(_) => HttpResponse::NotFound().body("Scrap not found"),
    }
}

pub async fn delete_scrap(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<IdPayload>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match Scrap::find_by_id(&conn, data.id) {
        Ok(scrap) => {
            if let Err(e) = scrap.delete(&conn) { return HttpResponse::InternalServerError().body(e.to_string()); }
            HttpResponse::Ok().body("Scrap deleted successfully")
        }
        Err(_) => HttpResponse::NotFound().body("Scrap not found")
    }
}

pub async fn all_scraps(conn_data: web::Data<Pool<SqliteConnectionManager>>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match Scrap::all(&conn) {
        Ok(scraps) => HttpResponse::Ok().json(scraps),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn filter_scraps(
    conn_data: web::Data<Pool<SqliteConnectionManager>>,
    web::Query(filter): web::Query<ScrapFilterPayload>
) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match Scrap::filter(&conn, &filter) {
        Ok(scraps) => HttpResponse::Ok().json(scraps),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}
