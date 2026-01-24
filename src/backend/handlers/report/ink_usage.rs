use actix_web::{web, HttpResponse, Responder};
use r2d2_sqlite::SqliteConnectionManager;
use r2d2::Pool;
use crate::backend::models::{ActualInkUsage, ActualInkUsageCreatePayload, ActualInkUsagePayload, ActualInkUsageFilterPayload, IdPayload};

pub async fn create_ink_usage(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<ActualInkUsageCreatePayload>, session: actix_session::Session) -> impl Responder {
    let user_id: Option<i32> = session.get("user_id").unwrap_or(None);
    if user_id.is_none() {
        return HttpResponse::Unauthorized().body("User not authenticated");
    }
    
    let conn = conn_data.get().unwrap();
    match ActualInkUsage::create(&conn, &data, user_id.unwrap()) {
        Ok(ink_usage) => HttpResponse::Ok().json(ink_usage),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn update_ink_usage(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<ActualInkUsagePayload>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match ActualInkUsage::find_by_id(&conn, data.id) {
        Ok(mut ink_usage) => {
            if let Err(e) = ink_usage.update(&conn, &data) { return HttpResponse::InternalServerError().body(e.to_string()); }
            HttpResponse::Ok().json(ink_usage)
        }
        Err(_) => HttpResponse::NotFound().body("Ink usage not found"),
    }
}

pub async fn delete_ink_usage(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<IdPayload>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match ActualInkUsage::find_by_id(&conn, data.id) {
        Ok(ink_usage) => {
            if let Err(e) = ink_usage.delete(&conn) { return HttpResponse::InternalServerError().body(e.to_string()); }
            HttpResponse::Ok().body("Ink usage deleted successfully")
        }
        Err(_) => HttpResponse::NotFound().body("Ink usage not found")
    }
}

pub async fn all_ink_usages(conn_data: web::Data<Pool<SqliteConnectionManager>>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match ActualInkUsage::all(&conn) {
        Ok(ink_usages) => HttpResponse::Ok().json(ink_usages),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn filter_ink_usages(
    conn_data: web::Data<Pool<SqliteConnectionManager>>,
    web::Query(filter): web::Query<ActualInkUsageFilterPayload>
) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match ActualInkUsage::filter(&conn, &filter) {
        Ok(ink_usages) => HttpResponse::Ok().json(ink_usages),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}
