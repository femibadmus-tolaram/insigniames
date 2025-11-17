use actix_web::{web, HttpResponse, Responder};
use r2d2_sqlite::SqliteConnectionManager;
use r2d2::Pool;
use crate::backend::models::{Machine, MachineCreatePayload, MachinePayload, IdPayload, MachineFilterPayload};

pub async fn create_machine(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<MachineCreatePayload>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match Machine::create(&conn, &data) {
        Ok(machine) => HttpResponse::Ok().json(machine),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn update_machine(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<MachinePayload>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match Machine::find_by_id(&conn, data.id) {
        Ok(mut machine) => {
            if let Err(e) = machine.update(&conn, &data) { return HttpResponse::InternalServerError().body(e.to_string()); }
            HttpResponse::Ok().json(machine)
        }
        Err(_) => HttpResponse::NotFound().body("Machine not found"),
    }
}

pub async fn delete_machine(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<IdPayload>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match Machine::find_by_id(&conn, data.id) {
        Ok(machine) => {
            match Machine::has_jobs(&conn, data.id) {
                Ok(true) => return HttpResponse::BadRequest().body("Cannot delete machine with existing jobs"),
                Ok(false) => {
                    if let Err(e) = machine.delete(&conn) { return HttpResponse::InternalServerError().body(e.to_string()); }
                    HttpResponse::Ok().body("Machine deleted successfully")
                }
                Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
            }
        }
        Err(_) => HttpResponse::NotFound().body("Machine not found")
    }
}

pub async fn all_machines(conn_data: web::Data<Pool<SqliteConnectionManager>>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match Machine::all(&conn) {
        Ok(machines) => HttpResponse::Ok().json(machines),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn filter_machines(
    conn_data: web::Data<Pool<SqliteConnectionManager>>,
    web::Query(filter): web::Query<MachineFilterPayload>
) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match Machine::filter(&conn, &filter) {
        Ok(machines) => HttpResponse::Ok().json(machines),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}
