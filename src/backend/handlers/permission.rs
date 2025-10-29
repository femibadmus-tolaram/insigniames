use actix_web::{web, HttpResponse, Responder};
use r2d2_sqlite::SqliteConnectionManager;
use r2d2::Pool;
use crate::backend::models::{Permission, PermissionPayload, PermissionUpdatePayload, IdPayload};

pub async fn create_permission(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<PermissionPayload>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match Permission::create(&conn, &data) { Ok(p) => HttpResponse::Ok().json(p), Err(e) => HttpResponse::InternalServerError().body(e.to_string()) }
}

pub async fn update_permission(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<PermissionUpdatePayload>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    let perm_list = Permission::all(&conn).unwrap();
    if let Some(mut p) = perm_list.into_iter().find(|p| p.id == data.id) {
        p.update(&conn, &data).unwrap();
        HttpResponse::Ok().json(p)
    } else { HttpResponse::NotFound().body("Permission not found") }
}

pub async fn delete_permission(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<IdPayload>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match Permission::find_by_id(&conn, data.id) {
        Ok(found) => {
            if !found {
                return HttpResponse::NotFound().body("Permission not found.");
            }
            match Permission::count_linked_roles(&conn, data.id) {
                Ok(count) => {
                    if count > 0 {
                        return HttpResponse::Conflict().body("Cannot delete permission. It is linked to one or more roles.");
                    }
                },
                Err(e) => {
                    return HttpResponse::InternalServerError().body(e.to_string());
                }
            }
            if let Err(e) = Permission::delete(&conn, data.id) {
                return HttpResponse::InternalServerError().body(e.to_string());
            }
            HttpResponse::Ok().body("Permission deleted successfully.")
        }
        Err(e) => {
            HttpResponse::InternalServerError().body(e.to_string())
        }
    }
}

pub async fn all_permissions(conn_data: web::Data<Pool<SqliteConnectionManager>>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match Permission::all(&conn) { Ok(list) => HttpResponse::Ok().json(list), Err(e) => HttpResponse::InternalServerError().body(e.to_string()) }
}
