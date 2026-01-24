use crate::backend::models::{
    EndInputRollPayload, IdPayload, InputRoll, InputRollCreatePayload, InputRollFilterPayload,
    InputRollUpdatePayload,
};
use actix_web::{HttpResponse, Responder, web};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

pub async fn create_input_roll(
    conn_data: web::Data<Pool<SqliteConnectionManager>>,
    data: web::Json<InputRollCreatePayload>,
    session: actix_session::Session,
) -> impl Responder {
    let user_id: Option<i32> = session.get("user_id").unwrap_or(None);
    if user_id.is_none() {
        return HttpResponse::Unauthorized().body("User not authenticated");
    }
    let conn = conn_data.get().unwrap();
    match InputRoll::create(&conn, &data, user_id.unwrap()) {
        Ok(roll) => HttpResponse::Ok().json(roll),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn update_input_roll(
    conn_data: web::Data<Pool<SqliteConnectionManager>>,
    data: web::Json<InputRollUpdatePayload>,
) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match InputRoll::find_by_id(&conn, data.id) {
        Ok(mut roll) => {
            if let Err(e) = roll.update(&conn, &data) {
                return HttpResponse::InternalServerError().body(e.to_string());
            }
            HttpResponse::Ok().json(roll)
        }
        Err(_) => HttpResponse::NotFound().body("Input roll not found"),
    }
}

pub async fn delete_input_roll(
    conn_data: web::Data<Pool<SqliteConnectionManager>>,
    data: web::Json<IdPayload>,
) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match InputRoll::find_by_id(&conn, data.id) {
        Ok(roll) => {
            if let Err(e) = roll.delete(&conn) {
                return HttpResponse::InternalServerError().body(e.to_string());
            }
            HttpResponse::Ok().body("Input roll deleted successfully")
        }
        Err(_) => HttpResponse::NotFound().body("Input roll not found"),
    }
}

pub async fn all_input_rolls(
    conn_data: web::Data<Pool<SqliteConnectionManager>>,
) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match InputRoll::all(&conn) {
        Ok(rolls) => HttpResponse::Ok().json(rolls),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn filter_input_rolls(
    conn_data: web::Data<Pool<SqliteConnectionManager>>,
    web::Query(filter): web::Query<InputRollFilterPayload>,
) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match InputRoll::filter(&conn, &filter) {
        Ok(rolls) => HttpResponse::Ok().json(rolls),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn input_roll_details(
    conn_data: web::Data<Pool<SqliteConnectionManager>>,
    web::Query(data): web::Query<IdPayload>,
) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match InputRoll::find_by_id(&conn, data.id) {
        Ok(details) => HttpResponse::Ok().json(details),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn end_input_roll(
    conn_data: web::Data<Pool<SqliteConnectionManager>>,
    data: web::Json<EndInputRollPayload>,
) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match InputRoll::end_input_roll(&conn, &data).await {
        Ok(document_number) => HttpResponse::Ok().json(document_number),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn filter_input_rolls_with_stats(
    conn_data: web::Data<Pool<SqliteConnectionManager>>,
    web::Query(filter): web::Query<InputRollFilterPayload>,
) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match InputRoll::filter(&conn, &filter) {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}
