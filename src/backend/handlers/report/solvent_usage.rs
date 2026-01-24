use actix_web::{web, HttpResponse, Responder};
use r2d2_sqlite::SqliteConnectionManager;
use r2d2::Pool;
use crate::backend::models::{ActualSolventUsage, ActualSolventUsageCreatePayload, ActualSolventUsagePayload, ActualSolventUsageFilterPayload, IdPayload};

pub async fn create_solvent_usage(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<ActualSolventUsageCreatePayload>, session: actix_session::Session) -> impl Responder {
    let user_id: Option<i32> = session.get("user_id").unwrap_or(None);
    if user_id.is_none() {
        return HttpResponse::Unauthorized().body("User not authenticated");
    }
    
    let conn = conn_data.get().unwrap();
    match ActualSolventUsage::create(&conn, &data, user_id.unwrap()) {
        Ok(solvent_usage) => HttpResponse::Ok().json(solvent_usage),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn update_solvent_usage(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<ActualSolventUsagePayload>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match ActualSolventUsage::find_by_id(&conn, data.id) {
        Ok(mut solvent_usage) => {
            if let Err(e) = solvent_usage.update(&conn, &data) { return HttpResponse::InternalServerError().body(e.to_string()); }
            HttpResponse::Ok().json(solvent_usage)
        }
        Err(_) => HttpResponse::NotFound().body("Solvent usage not found"),
    }
}

pub async fn delete_solvent_usage(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<IdPayload>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match ActualSolventUsage::find_by_id(&conn, data.id) {
        Ok(solvent_usage) => {
            if let Err(e) = solvent_usage.delete(&conn) { return HttpResponse::InternalServerError().body(e.to_string()); }
            HttpResponse::Ok().body("Solvent usage deleted successfully")
        }
        Err(_) => HttpResponse::NotFound().body("Solvent usage not found")
    }
}

pub async fn all_solvent_usages(conn_data: web::Data<Pool<SqliteConnectionManager>>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match ActualSolventUsage::all(&conn) {
        Ok(solvent_usages) => HttpResponse::Ok().json(solvent_usages),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn filter_solvent_usages(
    conn_data: web::Data<Pool<SqliteConnectionManager>>,
    web::Query(filter): web::Query<ActualSolventUsageFilterPayload>
) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match ActualSolventUsage::filter(&conn, &filter) {
        Ok(solvent_usages) => HttpResponse::Ok().json(solvent_usages),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}
