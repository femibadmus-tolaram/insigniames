use actix_web::{web, HttpResponse, Responder};
use r2d2_sqlite::SqliteConnectionManager;
use r2d2::Pool;
use crate::backend::models::*;

pub async fn create_section(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<SectionCreatePayload>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match Section::create(&conn, &data) {
        Ok(section) => HttpResponse::Ok().json(section),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn update_section(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<SectionPayload>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match Section::find_by_id(&conn, data.id) {
        Ok(mut section) => {
            if let Err(e) = section.update(&conn, &data) { return HttpResponse::InternalServerError().body(e.to_string()); }
            HttpResponse::Ok().json(section)
        }
        Err(_) => HttpResponse::NotFound().body("Section not found"),
    }
}

pub async fn delete_section(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<IdPayload>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match Section::find_by_id(&conn, data.id) {
        Ok(section) => {
            match Section::has_machines(&conn, data.id) {
                Ok(true) => return HttpResponse::BadRequest().body("Cannot delete section with existing machines"),
                Ok(false) => {
                    if let Err(e) = section.delete(&conn) { return HttpResponse::InternalServerError().body(e.to_string()); }
                    HttpResponse::Ok().body("Section deleted successfully")
                }
                Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
            }
        }
        Err(_) => HttpResponse::NotFound().body("Section not found")
    }
}

pub async fn all_sections(conn_data: web::Data<Pool<SqliteConnectionManager>>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match Section::all(&conn) {
        Ok(sections) => HttpResponse::Ok().json(sections),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn filter_sections(conn_data: web::Data<Pool<SqliteConnectionManager>>, web::Query(filter): web::Query<SectionFilterPayload>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match Section::filter(&conn, &filter) {
        Ok(sections) => HttpResponse::Ok().json(sections),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn update_po_code_sections(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<SectionPoCodesPayload>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match Section::find_by_id(&conn, data.id) {
        Ok(section) => {
            if let Err(e) = section.update_po_codes(&conn, &data) { return HttpResponse::InternalServerError().body(e.to_string()); }
            HttpResponse::Ok().json("PO codes updated successfully")
        }
        Err(_) => HttpResponse::NotFound().body("Section not found"),
    }
}


