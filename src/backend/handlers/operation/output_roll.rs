use crate::backend::models::{
    IdPayload, OutputRoll, OutputRollCreatePayload, OutputRollFilterPayload, OutputRollPayload,
};
use actix_web::{HttpResponse, Responder, web};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

pub async fn create_output_roll(
    conn_data: web::Data<Pool<SqliteConnectionManager>>,
    data: web::Json<OutputRollCreatePayload>,
    session: actix_session::Session,
) -> impl Responder {
    let user_id: Option<i32> = session.get("user_id").unwrap_or(None);
    if user_id.is_none() {
        return HttpResponse::Unauthorized().body("User not authenticated");
    }
    let mut conn = conn_data.get().unwrap();
    match OutputRoll::create(&mut conn, &data, user_id.unwrap()) {
        Ok(roll) => HttpResponse::Ok().json(roll),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn update_output_roll(
    conn_data: web::Data<Pool<SqliteConnectionManager>>,
    data: web::Json<OutputRollPayload>,
) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match OutputRoll::find_by_id(&conn, data.id) {
        Ok(mut roll) => {
            if let Err(e) = roll.update(&conn, &data).await {
                return HttpResponse::InternalServerError().body(e.to_string());
            }
            HttpResponse::Ok().json(roll)
        }
        Err(_) => HttpResponse::NotFound().body("Output roll not found"),
    }
}

pub async fn delete_output_roll(
    conn_data: web::Data<Pool<SqliteConnectionManager>>,
    data: web::Json<IdPayload>,
) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match OutputRoll::find_by_id(&conn, data.id) {
        Ok(roll) => {
            if let Err(e) = roll.delete(&conn) {
                return HttpResponse::InternalServerError().body(e.to_string());
            }
            HttpResponse::Ok().body("Output roll deleted successfully")
        }
        Err(_) => HttpResponse::NotFound().body("Output roll not found"),
    }
}

pub async fn all_output_rolls(
    conn_data: web::Data<Pool<SqliteConnectionManager>>,
) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match OutputRoll::all(&conn) {
        Ok(rolls) => HttpResponse::Ok().json(rolls),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn filter_output_rolls(
    conn_data: web::Data<Pool<SqliteConnectionManager>>,
    web::Query(filter): web::Query<OutputRollFilterPayload>,
) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match OutputRoll::filter(&conn, &filter) {
        Ok(rolls) => HttpResponse::Ok().json(rolls),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn output_roll_details(
    conn_data: web::Data<Pool<SqliteConnectionManager>>,
    web::Query(data): web::Query<IdPayload>,
) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match OutputRoll::get_details(&conn, data.id) {
        Ok(details) => HttpResponse::Ok().json(details),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}
