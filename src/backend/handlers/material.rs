use actix_web::{web, HttpResponse, Responder};
use r2d2_sqlite::SqliteConnectionManager;
use r2d2::Pool;
use crate::backend::models::{Material, MaterialCreatePayload, MaterialPayload, IdPayload, MaterialFilterPayload};

pub async fn create_material(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<MaterialCreatePayload>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match Material::create(&conn, &data) {
        Ok(material) => HttpResponse::Ok().json(material),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn update_material(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<MaterialPayload>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match Material::find_by_id(&conn, data.id) {
        Ok(mut material) => {
            if let Err(e) = material.update(&conn, &data) { return HttpResponse::InternalServerError().body(e.to_string()); }
            HttpResponse::Ok().json(material)
        }
        Err(_) => HttpResponse::NotFound().body("Material not found"),
    }
}

pub async fn delete_material(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<IdPayload>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match Material::find_by_id(&conn, data.id) {
        Ok(material) => {
            if let Err(e) = material.delete(&conn) { return HttpResponse::InternalServerError().body(e.to_string()); }
            HttpResponse::Ok().body("Material deleted successfully")
        }
        Err(_) => HttpResponse::NotFound().body("Material not found")
    }
}

pub async fn all_materials(conn_data: web::Data<Pool<SqliteConnectionManager>>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match Material::all(&conn) {
        Ok(materials) => HttpResponse::Ok().json(materials),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn filter_materials(
    conn_data: web::Data<Pool<SqliteConnectionManager>>,
    web::Query(filter): web::Query<MaterialFilterPayload>
) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match Material::filter(&conn, &filter) {
        Ok(materials) => HttpResponse::Ok().json(materials),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}