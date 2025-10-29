use actix_web::{web, HttpResponse, Responder};
use r2d2_sqlite::SqliteConnectionManager;
use r2d2::Pool;
use actix_session::Session;
use crate::backend::models::{User, UserPayload, UserCreatePayload, SigninPayload, IdPayload};


pub async fn create_user(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<UserCreatePayload>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match User::create(&conn, &data) {
        Ok(u) => HttpResponse::Ok().json(u),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn update_user(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<UserPayload>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match User::find_by_id(&conn, data.id) {
        Ok(mut u) => {
            if let Err(e) = u.update(&conn, &data) { return HttpResponse::InternalServerError().body(e.to_string()); }
            HttpResponse::Ok().json(u)
        }
        Err(_) => HttpResponse::NotFound().body("User not found"),
    }
}

pub async fn delete_user(conn_data: web::Data<Pool<SqliteConnectionManager>>, data: web::Json<IdPayload>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match User::find_by_id(&conn, data.id) {
        Ok(u) => {
            match User::count_linked_records(&conn, data.id) {
                Ok(count) => {
                    if count > 0 {
                        return HttpResponse::Conflict().body("Cannot delete user. It is linked to weigh logs, contractors, or plans.");
                    }
                },
                Err(e) => {
                    return HttpResponse::InternalServerError().body(e.to_string());
                }
            }
            if let Err(e) = u.delete(&conn) {
                return HttpResponse::InternalServerError().body(e.to_string());
            }
            HttpResponse::Ok().body("User deleted successfully.")
        }
        Err(_) => HttpResponse::NotFound().body("User not found.")
    }
}

pub async fn all_users(conn_data: web::Data<Pool<SqliteConnectionManager>>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match User::all(&conn) {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn signin_user(conn_data: web::Data<Pool<SqliteConnectionManager>>, session: Session, data: web::Json<SigninPayload>) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match User::signin(&conn, &data) {
        Ok(user) => {
            session.insert("user_id", user.id.clone()).unwrap();
            session.insert("user_name", user.whois.clone()).unwrap();
            HttpResponse::Ok().json(user)
        }
        Err(_) => HttpResponse::Unauthorized().body("Invalid credentials"),
    }
}

pub async fn get_me(conn_data: web::Data<Pool<SqliteConnectionManager>>, session: Session) -> impl Responder {
    let conn = conn_data.get().unwrap();
    match session.get::<i32>("user_id") {
        Ok(Some(user_id)) => match User::me(&conn, user_id) {
            Ok(user) => HttpResponse::Ok().json(user),
            Err(_) => HttpResponse::InternalServerError().body("Failed to fetch user"),
        },
        Ok(None) | Err(_) => HttpResponse::Unauthorized().body("Invalid session"),
    }
}

pub async fn signout_user(session: Session) -> impl Responder {
    session.remove("user_id");
    HttpResponse::Ok().body("Signed out")
}

