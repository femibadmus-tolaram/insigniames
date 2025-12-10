use actix_web::{web, HttpResponse, Responder};
use r2d2_sqlite::SqliteConnectionManager;
use r2d2::Pool;
use crate::backend::models::{ProcessOrder, ProcessOrderFilterPayload};

pub async fn get_process_orders(
    conn_data: web::Data<Pool<SqliteConnectionManager>>,
    web::Query(filter): web::Query<ProcessOrderFilterPayload>
) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match ProcessOrder::filter(&conn, &filter) {
        Ok(orders) => HttpResponse::Ok().json(orders),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn get_all_process_orders(
    conn_data: web::Data<Pool<SqliteConnectionManager>>
) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match ProcessOrder::all(&conn) {
        Ok(orders) => HttpResponse::Ok().json(orders),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}