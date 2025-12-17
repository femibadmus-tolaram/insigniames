use actix_web::{web, HttpResponse, Responder};
use r2d2_sqlite::SqliteConnectionManager;
use r2d2::Pool;
use crate::backend::models::*;


pub async fn create_shift(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<LookupCreatePayload>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match Shift::create(&conn, &data) {
        Ok(shift) => HttpResponse::Ok().json(shift),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn update_shift(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<LookupPayload>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match Shift::find_by_id(&conn, data.id) {
        Ok(mut shift) => {
            if let Err(e) = shift.update(&conn, &data) { return HttpResponse::InternalServerError().body(e.to_string()); }
            HttpResponse::Ok().json(shift)
        }
        Err(_) => HttpResponse::NotFound().body("Shift not found"),
    }
}

pub async fn delete_shift(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<IdPayload>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match Shift::find_by_id(&conn, data.id) {
        Ok(shift) => {
            match Shift::has_related_records(&conn, data.id) {
                Ok(true) => return HttpResponse::BadRequest().body("Cannot delete shift with existing records"),
                Ok(false) => {
                    if let Err(e) = shift.delete(&conn) { return HttpResponse::InternalServerError().body(e.to_string()); }
                    HttpResponse::Ok().body("Shift deleted successfully")
                }
                Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
            }
        }
        Err(_) => HttpResponse::NotFound().body("Shift not found")
    }
}

pub async fn all_shifts(conn_data: web::Data<Pool<SqliteConnectionManager>>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match Shift::all(&conn) {
        Ok(shifts) => HttpResponse::Ok().json(shifts),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}


pub async fn create_colour(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<LookupCreatePayload>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match Colour::create(&conn, &data) {
        Ok(colour) => HttpResponse::Ok().json(colour),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn update_colour(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<LookupPayload>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match Colour::find_by_id(&conn, data.id) {
        Ok(mut colour) => {
            if let Err(e) = colour.update(&conn, &data) { return HttpResponse::InternalServerError().body(e.to_string()); }
            HttpResponse::Ok().json(colour)
        }
        Err(_) => HttpResponse::NotFound().body("Colour not found"),
    }
}

pub async fn delete_colour(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<IdPayload>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match Colour::find_by_id(&conn, data.id) {
        Ok(colour) => {
            match Colour::has_related_records(&conn, data.id) {
                Ok(true) => return HttpResponse::BadRequest().body("Cannot delete colour with existing ink usage records"),
                Ok(false) => {
                    if let Err(e) = colour.delete(&conn) { return HttpResponse::InternalServerError().body(e.to_string()); }
                    HttpResponse::Ok().body("Colour deleted successfully")
                }
                Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
            }
        }
        Err(_) => HttpResponse::NotFound().body("Colour not found")
    }
}

pub async fn all_colours(conn_data: web::Data<Pool<SqliteConnectionManager>>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match Colour::all(&conn) {
        Ok(colours) => HttpResponse::Ok().json(colours),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}


pub async fn create_solvent_type(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<LookupCreatePayload>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match SolventType::create(&conn, &data) {
        Ok(solvent_type) => HttpResponse::Ok().json(solvent_type),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn update_solvent_type(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<LookupPayload>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match SolventType::find_by_id(&conn, data.id) {
        Ok(mut solvent_type) => {
            if let Err(e) = solvent_type.update(&conn, &data) { return HttpResponse::InternalServerError().body(e.to_string()); }
            HttpResponse::Ok().json(solvent_type)
        }
        Err(_) => HttpResponse::NotFound().body("Solvent type not found"),
    }
}

pub async fn delete_solvent_type(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<IdPayload>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match SolventType::find_by_id(&conn, data.id) {
        Ok(solvent_type) => {
            match SolventType::has_related_records(&conn, data.id) {
                Ok(true) => return HttpResponse::BadRequest().body("Cannot delete solvent type with existing usage records"),
                Ok(false) => {
                    if let Err(e) = solvent_type.delete(&conn) { return HttpResponse::InternalServerError().body(e.to_string()); }
                    HttpResponse::Ok().body("Solvent type deleted successfully")
                }
                Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
            }
        }
        Err(_) => HttpResponse::NotFound().body("Solvent type not found")
    }
}


pub async fn all_solvent_types(conn_data: web::Data<Pool<SqliteConnectionManager>>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match SolventType::all(&conn) {
        Ok(solvent_types) => HttpResponse::Ok().json(solvent_types),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn create_scrap_type(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<LookupCreatePayload>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match ScrapType::create(&conn, &data) {
        Ok(scrap_type) => HttpResponse::Ok().json(scrap_type),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn update_scrap_type(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<LookupPayload>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match ScrapType::find_by_id(&conn, data.id) {
        Ok(mut scrap_type) => {
            if let Err(e) = scrap_type.update(&conn, &data) { return HttpResponse::InternalServerError().body(e.to_string()); }
            HttpResponse::Ok().json(scrap_type)
        }
        Err(_) => HttpResponse::NotFound().body("Scrap type not found"),
    }
}

pub async fn delete_scrap_type(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<IdPayload>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match ScrapType::find_by_id(&conn, data.id) {
        Ok(scrap_type) => {
            match ScrapType::has_related_records(&conn, data.id) {
                Ok(true) => return HttpResponse::BadRequest().body("Cannot delete scrap type with existing scrap records"),
                Ok(false) => {
                    if let Err(e) = scrap_type.delete(&conn) { return HttpResponse::InternalServerError().body(e.to_string()); }
                    HttpResponse::Ok().body("Scrap type deleted successfully")
                }
                Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
            }
        }
        Err(_) => HttpResponse::NotFound().body("Scrap type not found")
    }
}

pub async fn all_scrap_types(conn_data: web::Data<Pool<SqliteConnectionManager>>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match ScrapType::all(&conn) {
        Ok(scrap_types) => HttpResponse::Ok().json(scrap_types),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}


pub async fn create_downtime_reason(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<LookupCreatePayload>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match DowntimeReason::create(&conn, &data) {
        Ok(downtime_reason) => HttpResponse::Ok().json(downtime_reason),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn update_downtime_reason(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<LookupPayload>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match DowntimeReason::find_by_id(&conn, data.id) {
        Ok(mut downtime_reason) => {
            if let Err(e) = downtime_reason.update(&conn, &data) { return HttpResponse::InternalServerError().body(e.to_string()); }
            HttpResponse::Ok().json(downtime_reason)
        }
        Err(_) => HttpResponse::NotFound().body("Downtime reason not found"),
    }
}

pub async fn delete_downtime_reason(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<IdPayload>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match DowntimeReason::find_by_id(&conn, data.id) {
        Ok(downtime_reason) => {
            match DowntimeReason::has_related_records(&conn, data.id) {
                Ok(true) => return HttpResponse::BadRequest().body("Cannot delete downtime reason with existing downtime records"),
                Ok(false) => {
                    if let Err(e) = downtime_reason.delete(&conn) { return HttpResponse::InternalServerError().body(e.to_string()); }
                    HttpResponse::Ok().body("Downtime reason deleted successfully")
                }
                Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
            }
        }
        Err(_) => HttpResponse::NotFound().body("Downtime reason not found")
    }
}

pub async fn all_downtime_reasons(conn_data: web::Data<Pool<SqliteConnectionManager>>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match DowntimeReason::all(&conn) {
        Ok(downtime_reasons) => HttpResponse::Ok().json(downtime_reasons),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}


pub async fn create_flag_reason(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<LookupCreatePayload>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match FlagReason::create(&conn, &data) {
        Ok(flag_reason) => HttpResponse::Ok().json(flag_reason),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn update_flag_reason(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<LookupPayload>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match FlagReason::find_by_id(&conn, data.id) {
        Ok(mut flag_reason) => {
            if let Err(e) = flag_reason.update(&conn, &data) { return HttpResponse::InternalServerError().body(e.to_string()); }
            HttpResponse::Ok().json(flag_reason)
        }
        Err(_) => HttpResponse::NotFound().body("Flag reason not found"),
    }
}

pub async fn delete_flag_reason(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<IdPayload>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match FlagReason::find_by_id(&conn, data.id) {
        Ok(flag_reason) => {
            match FlagReason::has_related_records(&conn, data.id) {
                Ok(true) => return HttpResponse::BadRequest().body("Cannot delete flag reason with existing roll records"),
                Ok(false) => {
                    if let Err(e) = flag_reason.delete(&conn) { return HttpResponse::InternalServerError().body(e.to_string()); }
                    HttpResponse::Ok().body("Flag reason deleted successfully")
                }
                Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
            }
        }
        Err(_) => HttpResponse::NotFound().body("Flag reason not found")
    }
}

pub async fn all_flag_reasons(conn_data: web::Data<Pool<SqliteConnectionManager>>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match FlagReason::all(&conn) {
        Ok(flag_reasons) => HttpResponse::Ok().json(flag_reasons),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}


pub async fn add_po_code_section(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<POCodeSectionPayload>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match POCodeSection::create(&conn, data.po_code_id, data.section_id) {
        Ok(_) => HttpResponse::Ok().body("POCode section added successfully"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn remove_po_code_section(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<POCodeSectionPayload>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match POCodeSection::delete(&conn, data.po_code_id, data.section_id) {
        Ok(_) => HttpResponse::Ok().body("POCode section removed successfully"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn get_po_code_sections(conn_data: web::Data<Pool<SqliteConnectionManager>>, info: web::Path<i32>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match POCodeSection::find_by_po_code(&conn, info.into_inner()) {
        Ok(sections) => HttpResponse::Ok().json(sections),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}


pub async fn create_po_code(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<LookupCreatePayload>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match POCode::create(&conn, &data) {
        Ok(po_code) => HttpResponse::Ok().json(po_code),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn update_po_code(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<LookupPayload>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match POCode::find_by_id(&conn, data.id) {
        Ok(mut po_code) => {
            if let Err(e) = po_code.update(&conn, &data) { return HttpResponse::InternalServerError().body(e.to_string()); }
            HttpResponse::Ok().json(po_code)
        }
        Err(_) => HttpResponse::NotFound().body("PO Code not found"),
    }
}

pub async fn delete_po_code(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<IdPayload>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match POCode::find_by_id(&conn, data.id) {
        Ok(po_code) => {
            match POCode::has_related_records(&conn, data.id) {
                Ok(true) => return HttpResponse::BadRequest().body("Cannot delete PO Code with existing records"),
                Ok(false) => {
                    if let Err(e) = po_code.delete(&conn) { return HttpResponse::InternalServerError().body(e.to_string()); }
                    HttpResponse::Ok().body("PO Code deleted successfully")
                }
                Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
            }
        }
        Err(_) => HttpResponse::NotFound().body("PO Code not found")
    }
}

pub async fn all_po_codes(conn_data: web::Data<Pool<SqliteConnectionManager>>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match POCode::all(&conn) {
        Ok(po_codes) => HttpResponse::Ok().json(po_codes),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}


