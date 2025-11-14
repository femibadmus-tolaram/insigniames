use actix_web::{web, HttpResponse, Responder};
use r2d2_sqlite::SqliteConnectionManager;
use r2d2::Pool;
use crate::backend::models::{Downtime, DowntimeCreatePayload, DowntimePayload, IdPayload, DowntimeFilterPayload};

pub async fn create_downtime(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<DowntimeCreatePayload>, session: actix_session::Session) -> impl Responder {
    let user_id: Option<i32> = session.get("user_id").unwrap_or(None);
    if user_id.is_none() {
        return HttpResponse::Unauthorized().body("User not authenticated");
    }
    
    let conn = conn_data.get().unwrap();
    match Downtime::create(&conn, &data, user_id.unwrap()) {
        Ok(downtime) => HttpResponse::Ok().json(downtime),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn update_downtime(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<DowntimePayload>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match Downtime::find_by_id(&conn, data.id) {
        Ok(mut downtime) => {
            if let Err(e) = downtime.update(&conn, &data) { return HttpResponse::InternalServerError().body(e.to_string()); }
            HttpResponse::Ok().json(downtime)
        }
        Err(_) => HttpResponse::NotFound().body("Downtime not found"),
    }
}

pub async fn delete_downtime(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<IdPayload>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match Downtime::find_by_id(&conn, data.id) {
        Ok(downtime) => {
            if let Err(e) = downtime.delete(&conn) { return HttpResponse::InternalServerError().body(e.to_string()); }
            HttpResponse::Ok().body("Downtime deleted successfully")
        }
        Err(_) => HttpResponse::NotFound().body("Downtime not found")
    }
}

pub async fn all_downtimes(conn_data: web::Data<Pool<SqliteConnectionManager>>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match Downtime::all(&conn) {
        Ok(downtimes) => HttpResponse::Ok().json(downtimes),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn filter_downtimes(
    conn_data: web::Data<Pool<SqliteConnectionManager>>,
    web::Query(filter): web::Query<DowntimeFilterPayload>
) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match Downtime::filter(&conn, &filter) {
        Ok(downtimes) => HttpResponse::Ok().json(downtimes),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}
