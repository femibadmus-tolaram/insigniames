use actix_web::{web, HttpResponse, Responder};
use r2d2_sqlite::SqliteConnectionManager;
use r2d2::Pool;
use crate::backend::models::{Shift, Colour, SolventType, ScrapType, DowntimeReason, FlagReason, LookupCreatePayload, LookupPayload, IdPayload, ManufacturingOrderType};


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


pub async fn create_manufacturing_order_type(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<LookupCreatePayload>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match ManufacturingOrderType::create(&conn, &data) {
        Ok(manufacturing_order_type) => HttpResponse::Ok().json(manufacturing_order_type),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn update_manufacturing_order_type(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<LookupPayload>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match ManufacturingOrderType::find_by_id(&conn, data.id) {
        Ok(mut manufacturing_order_type) => {
            if let Err(e) = manufacturing_order_type.update(&conn, &data) { return HttpResponse::InternalServerError().body(e.to_string()); }
            HttpResponse::Ok().json(manufacturing_order_type)
        }
        Err(_) => HttpResponse::NotFound().body("Order type not found"),
    }
}

pub async fn delete_manufacturing_order_type(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<IdPayload>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match ManufacturingOrderType::find_by_id(&conn, data.id) {
        Ok(manufacturing_order_type) => {
            match ManufacturingOrderType::has_related_records(&conn, data.id) {
                Ok(true) => return HttpResponse::BadRequest().body("Cannot delete Order type with existing records"),
                Ok(false) => {
                    if let Err(e) = manufacturing_order_type.delete(&conn) { return HttpResponse::InternalServerError().body(e.to_string()); }
                    HttpResponse::Ok().body("Order type deleted successfully")
                }
                Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
            }
        }
        Err(_) => HttpResponse::NotFound().body("Order type not found")
    }
}

pub async fn all_manufacturing_order_type(conn_data: web::Data<Pool<SqliteConnectionManager>>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match ManufacturingOrderType::all(&conn) {
        Ok(manufacturing_order_types) => HttpResponse::Ok().json(manufacturing_order_types),
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

