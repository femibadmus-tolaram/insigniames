use actix_web::{web, HttpResponse, Responder};
use r2d2_sqlite::SqliteConnectionManager;
use r2d2::Pool;
use crate::backend::models::{Roll, RollCreatePayload, RollPayload, IdPayload, RollFilterPayload};

pub async fn create_roll(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<RollCreatePayload>, session: actix_session::Session) -> impl Responder {
    let user_id: Option<i32> = session.get("user_id").unwrap_or(None);
    if user_id.is_none() {
        return HttpResponse::Unauthorized().body("User not authenticated");
    }
    
    let conn = conn_data.get().unwrap();
    match Roll::create(&conn, &data, user_id.unwrap()) {
        Ok(roll) => HttpResponse::Ok().json(roll),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn update_roll(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<RollPayload>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match Roll::find_by_id(&conn, data.id) {
        Ok(mut roll) => {
            if let Err(e) = roll.update(&conn, &data) { return HttpResponse::InternalServerError().body(e.to_string()); }
            HttpResponse::Ok().json(roll)
        }
        Err(_) => HttpResponse::NotFound().body("Roll not found"),
    }
}

pub async fn delete_roll(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<IdPayload>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match Roll::find_by_id(&conn, data.id) {
        Ok(roll) => {
            if let Err(e) = roll.delete(&conn) { return HttpResponse::InternalServerError().body(e.to_string()); }
            HttpResponse::Ok().body("Roll deleted successfully")
        }
        Err(_) => HttpResponse::NotFound().body("Roll not found")
    }
}

pub async fn all_rolls(conn_data: web::Data<Pool<SqliteConnectionManager>>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match Roll::all(&conn) {
        Ok(rolls) => HttpResponse::Ok().json(rolls),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn filter_rolls(
    conn_data: web::Data<Pool<SqliteConnectionManager>>,
    web::Query(filter): web::Query<RollFilterPayload>
) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match Roll::filter(&conn, &filter) {
        Ok(rolls) => HttpResponse::Ok().json(rolls),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}
