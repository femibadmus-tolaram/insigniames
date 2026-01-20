use crate::backend::models::{ProcessOrder, ProcessOrderFilterPayload};
use crate::sap::{materials, pso};
use actix_web::{HttpResponse, Responder, web};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

pub async fn get_process_orders(
    conn_data: web::Data<Pool<SqliteConnectionManager>>,
    web::Query(filter): web::Query<ProcessOrderFilterPayload>,
) -> impl Responder {
    let conn = conn_data.get().unwrap();

    match ProcessOrder::filter(&conn, &filter) {
        Ok(orders_response) => {
            let should_resync = orders_response.total_count == 0;

            if should_resync {
                if let Err(e) = pso::sync_process_orders(&conn_data).await {
                    eprintln!("Failed to sync process orders: {}", e);
                } else {
                    if let Err(e) = materials::sync_material_codes(&conn_data).await {
                        eprintln!("Failed to sync material codes: {}", e);
                    }
                }

                match ProcessOrder::filter(&conn, &filter) {
                    Ok(orders) => HttpResponse::Ok().json(orders),
                    Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
                }
            } else {
                HttpResponse::Ok().json(orders_response)
            }
        }
        Err(e) => {
            if let Err(_) = pso::sync_process_orders(&conn_data).await {
                return HttpResponse::InternalServerError().body(e.to_string());
            }

            if let Err(e) = materials::sync_material_codes(&conn_data).await {
                eprintln!("Failed to sync material codes: {}", e);
            }

            match ProcessOrder::filter(&conn, &filter) {
                Ok(orders) => HttpResponse::Ok().json(orders),
                Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
            }
        }
    }
}

pub async fn get_all_process_orders(
    conn_data: web::Data<Pool<SqliteConnectionManager>>,
) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match ProcessOrder::all(&conn) {
        Ok(orders) => HttpResponse::Ok().json(orders),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}
