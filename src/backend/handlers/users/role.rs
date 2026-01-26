use crate::backend::models::{IdPayload, Role, RolePayload, RoleUpdatePayload};
use actix_web::{HttpResponse, Responder, web};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

pub async fn create_role(
    conn_data: web::Data<Pool<SqliteConnectionManager>>,
    data: web::Json<RolePayload>,
) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match Role::create(&conn, &data) {
        Ok(r) => HttpResponse::Ok().json(r),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn update_role(
    conn_data: web::Data<Pool<SqliteConnectionManager>>,
    data: web::Json<RoleUpdatePayload>,
) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match Role::find_by_id(&conn, data.id) {
        Ok(mut r) => {
            r.update(&conn, &data).unwrap();
            HttpResponse::Ok().json(r)
        }
        Err(_) => HttpResponse::NotFound().body("Role not found"),
    }
}

pub async fn delete_role(
    conn_data: web::Data<Pool<SqliteConnectionManager>>,
    data: web::Json<IdPayload>,
) -> impl Responder {
    let mut conn = conn_data.get().unwrap();
    match Role::find_by_id(&mut conn, data.id) {
        Ok(r) => {
            if let Err(e) = r.delete(&mut conn) {
                return HttpResponse::InternalServerError().body(e.to_string());
            }
            HttpResponse::Ok().body("Role deleted successfully.")
        }
        Err(_) => HttpResponse::NotFound().body("Role not found."),
    }
}

pub async fn all_roles(conn_data: web::Data<Pool<SqliteConnectionManager>>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match Role::all(&conn) {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}
