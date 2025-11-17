use actix_web::{web, HttpResponse, Responder};
use r2d2_sqlite::SqliteConnectionManager;
use r2d2::Pool;
use crate::backend::models::{Job, JobCreatePayload, JobPayload, IdPayload, JobFilterPayload};

pub async fn create_job(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<JobCreatePayload>, session: actix_session::Session) -> impl Responder {
    let user_id: Option<i32> = session.get("user_id").unwrap_or(None);
    if user_id.is_none() {
        return HttpResponse::Unauthorized().body("User not authenticated");
    }
    
    let conn = conn_data.get().unwrap();
    match Job::create(&conn, &data, user_id.unwrap()) {
        Ok(job) => HttpResponse::Ok().json(job),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn update_job(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<JobPayload>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match Job::find_by_id(&conn, data.id) {
        Ok(mut job) => {
            if let Err(e) = job.update(&conn, &data) { return HttpResponse::InternalServerError().body(e.to_string()); }
            HttpResponse::Ok().json(job)
        }
        Err(_) => HttpResponse::NotFound().body("Job not found"),
    }
}

pub async fn delete_job(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<IdPayload>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match Job::find_by_id(&conn, data.id) {
        Ok(job) => {
            if let Err(e) = job.delete(&conn) { return HttpResponse::InternalServerError().body(e.to_string()); }
            HttpResponse::Ok().body("Job deleted successfully")
        }
        Err(_) => HttpResponse::NotFound().body("Job not found")
    }
}

pub async fn all_jobs(conn_data: web::Data<Pool<SqliteConnectionManager>>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match Job::all(&conn) {
        Ok(jobs) => HttpResponse::Ok().json(jobs),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn filter_jobs(
    conn_data: web::Data<Pool<SqliteConnectionManager>>,
    web::Query(filter): web::Query<JobFilterPayload>
) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match Job::filter(&conn, &filter) {
        Ok(jobs) => HttpResponse::Ok().json(jobs),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}
