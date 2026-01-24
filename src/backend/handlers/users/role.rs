use actix_web::{web, HttpResponse, Responder};
use r2d2_sqlite::SqliteConnectionManager;
use r2d2::Pool;
use crate::backend::models::{Role, RolePayload, RoleUpdatePayload, IdPayload};

pub async fn create_role(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<RolePayload>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match Role::create(&conn, &data) { Ok(r) => HttpResponse::Ok().json(r), Err(e) => HttpResponse::InternalServerError().body(e.to_string()) }
}

pub async fn update_role(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<RoleUpdatePayload>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match Role::find_by_id(&conn, data.id) {
        Ok(mut r) => { r.update(&conn, &data).unwrap(); HttpResponse::Ok().json(r) }
        Err(_) => HttpResponse::NotFound().body("Role not found")
    }
}

pub async fn delete_role(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<IdPayload>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match Role::find_by_id(&conn, data.id) {
        Ok(r) => {
            match Role::count_linked_records(&conn, data.id) {
                Ok(count) => {
                    if count > 0 {
                        return HttpResponse::Conflict().body("Cannot delete role. It is linked to users or permissions.");
                    }
                },
                Err(e) => {
                    return HttpResponse::InternalServerError().body(e.to_string());
                }
            }
            if let Err(e) = r.delete(&conn) {
                return HttpResponse::InternalServerError().body(e.to_string());
            }
            HttpResponse::Ok().body("Role deleted successfully.")
        }
        Err(_) => HttpResponse::NotFound().body("Role not found.")
    }
}

pub async fn all_roles(conn_data: web::Data<Pool<SqliteConnectionManager>>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match Role::all(&conn) { Ok(list) => HttpResponse::Ok().json(list), Err(e) => HttpResponse::InternalServerError().body(e.to_string()) }
}
