use actix_files::NamedFile;
use actix_multipart::Multipart;
use actix_session::Session;
use actix_web::{HttpRequest, HttpResponse, Responder, Result};
use askama::Template;
use futures_util::StreamExt as _;
use rand::distr::{Alphanumeric, SampleString};
use std::{fs::File, io::Write, path::Path};
use tokio::fs;

#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate {
    user_name: String,
}

#[derive(Template)]
#[template(path = "settings.html")]
struct SettingsTemplate;

#[derive(Template)]
#[template(path = "admin/users.html")]
struct UsersTemplate {
    user_name: String,
}

#[derive(Template)]
#[template(path = "admin/roles.html")]
struct RolesTemplate;

#[derive(Template)]
#[template(path = "report/downtime.html")]
struct DownTimeTemplate {
    user_name: String,
}

#[derive(Template)]
#[template(path = "report/consumable.html")]
struct ConsumableTemplate {
    user_name: String,
}
#[derive(Template)]
#[template(path = "report/scrap.html")]
struct ScrapTemplate {
    user_name: String,
}
#[derive(Template)]
#[template(path = "admin/lookup.html")]
struct LookUpTemplate {
    user_name: String,
}
#[derive(Template)]
#[template(path = "admin/machine.html")]
struct MachineTemplate {
    user_name: String,
}
#[derive(Template)]
#[template(path = "admin/material.html")]
struct MaterialTemplate {
    user_name: String,
}
#[derive(Template)]
#[template(path = "admin/section.html")]
struct SectionTemplate {
    user_name: String,
}
#[derive(Template)]
#[template(path = "production.html")]
struct ProductionTemplate {
    user_name: String,
}
#[derive(Template)]
#[template(path = "jobs.html")]
struct JobsTemplate {
    user_name: String,
}
#[derive(Template)]
#[template(path = "output-rolls.html")]
struct OutputRollsTemplate {
    user_name: String,
}
#[derive(Template)]
#[template(path = "input-rolls.html")]
struct InputRollsTemplate {
    user_name: String,
}

#[derive(Template)]
#[template(path = "signin.html")]
struct SigninTemplate;

pub async fn whois_data() -> impl Responder {
    let mut rng = rand::rng();
    let rand_str = Alphanumeric.sample_string(&mut rng, 30);
    let _ = fs::create_dir_all("data").await;
    let _ = fs::write("data/whois.txt", &rand_str).await;
    HttpResponse::Ok().content_type("text/plain").body(rand_str)
}

pub async fn roles_page() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(RolesTemplate.render().unwrap())
}

pub async fn downtime_page(session: Session) -> impl Responder {
    let user_name = session
        .get::<String>("user_name")
        .unwrap_or(None)
        .unwrap_or_default();
    HttpResponse::Ok()
        .content_type("text/html")
        .body(DownTimeTemplate { user_name }.render().unwrap())
}

pub async fn consumable_page(session: Session) -> impl Responder {
    let user_name = session
        .get::<String>("user_name")
        .unwrap_or(None)
        .unwrap_or_default();
    HttpResponse::Ok()
        .content_type("text/html")
        .body(ConsumableTemplate { user_name }.render().unwrap())
}

pub async fn scrap_page(session: Session) -> impl Responder {
    let user_name = session
        .get::<String>("user_name")
        .unwrap_or(None)
        .unwrap_or_default();
    HttpResponse::Ok()
        .content_type("text/html")
        .body(ScrapTemplate { user_name }.render().unwrap())
}

pub async fn lookup_page(session: Session) -> impl Responder {
    let user_name = session
        .get::<String>("user_name")
        .unwrap_or(None)
        .unwrap_or_default();
    HttpResponse::Ok()
        .content_type("text/html")
        .body(LookUpTemplate { user_name }.render().unwrap())
}

pub async fn machine_page(session: Session) -> impl Responder {
    let user_name = session
        .get::<String>("user_name")
        .unwrap_or(None)
        .unwrap_or_default();
    HttpResponse::Ok()
        .content_type("text/html")
        .body(MachineTemplate { user_name }.render().unwrap())
}

pub async fn material_page(session: Session) -> impl Responder {
    let user_name = session
        .get::<String>("user_name")
        .unwrap_or(None)
        .unwrap_or_default();
    HttpResponse::Ok()
        .content_type("text/html")
        .body(MaterialTemplate { user_name }.render().unwrap())
}

pub async fn section_page(session: Session) -> impl Responder {
    let user_name = session
        .get::<String>("user_name")
        .unwrap_or(None)
        .unwrap_or_default();
    HttpResponse::Ok()
        .content_type("text/html")
        .body(SectionTemplate { user_name }.render().unwrap())
}

pub async fn production_page(session: Session) -> impl Responder {
    let user_name = session
        .get::<String>("user_name")
        .unwrap_or(None)
        .unwrap_or_default();
    HttpResponse::Ok()
        .content_type("text/html")
        .body(ProductionTemplate { user_name }.render().unwrap())
}

pub async fn output_rolls_page(session: Session) -> impl Responder {
    let user_name = session
        .get::<String>("user_name")
        .unwrap_or(None)
        .unwrap_or_default();
    HttpResponse::Ok()
        .content_type("text/html")
        .body(OutputRollsTemplate { user_name }.render().unwrap())
}

pub async fn input_rolls_page(session: Session) -> impl Responder {
    let user_name = session
        .get::<String>("user_name")
        .unwrap_or(None)
        .unwrap_or_default();
    HttpResponse::Ok()
        .content_type("text/html")
        .body(InputRollsTemplate { user_name }.render().unwrap())
}

pub async fn jobs_page(session: Session) -> impl Responder {
    let user_name = session
        .get::<String>("user_name")
        .unwrap_or(None)
        .unwrap_or_default();
    HttpResponse::Ok()
        .content_type("text/html")
        .body(JobsTemplate { user_name }.render().unwrap())
}

pub async fn settings_page() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(SettingsTemplate.render().unwrap())
}

pub async fn home_page(session: Session) -> impl Responder {
    let user_name = session
        .get::<String>("user_name")
        .unwrap_or(None)
        .unwrap_or_default();
    HttpResponse::Ok()
        .content_type("text/html")
        .body(HomeTemplate { user_name }.render().unwrap())
}

pub async fn user_page(session: Session) -> impl Responder {
    let user_name = session
        .get::<String>("user_name")
        .unwrap_or(None)
        .unwrap_or_default();
    HttpResponse::Ok()
        .content_type("text/html")
        .body(UsersTemplate { user_name }.render().unwrap())
}

pub async fn signin_page(session: Session) -> impl Responder {
    if session.get::<String>("user_name").unwrap_or(None).is_some() {
        return HttpResponse::Found()
            .append_header(("Location", "/"))
            .finish();
    }
    HttpResponse::Ok()
        .content_type("text/html")
        .body(SigninTemplate.render().unwrap())
}

pub async fn logout(session: Session) -> impl Responder {
    session.remove("user_name");
    session.remove("user_id");
    HttpResponse::Found()
        .append_header(("Location", "/"))
        .finish()
}

pub async fn download_app(req: HttpRequest) -> Result<NamedFile> {
    let query: String = req.match_info().get("name").unwrap_or("").to_string();
    let file_path = format!("./static/release/{}", query);
    let file_path = if Path::new(&file_path).exists() {
        file_path
    } else {
        "./static/empty.txt".to_string()
    };
    Ok(NamedFile::open_async(file_path).await?)
}

pub async fn upload_app(req: HttpRequest, mut payload: Multipart) -> Result<HttpResponse> {
    let name = req.match_info().get("name").unwrap_or("").to_string();
    if name.is_empty() {
        return Ok(HttpResponse::BadRequest().body("Missing file name"));
    }

    let item = match payload.next().await {
        Some(Ok(field)) => field,
        Some(Err(_)) | None => return Ok(HttpResponse::BadRequest().body("No file uploaded")),
    };
    let mut field = item;

    let filepath = format!("./static/release/{}", name);
    let mut f = match File::create(&filepath) {
        Ok(f) => f,
        Err(_) => return Ok(HttpResponse::InternalServerError().body("Failed to create file")),
    };

    while let Some(chunk) = field.next().await {
        let data = match chunk {
            Ok(c) => c,
            Err(_) => {
                return Ok(HttpResponse::InternalServerError().body("Failed to read file chunk"));
            }
        };
        if f.write_all(&data).is_err() {
            return Ok(HttpResponse::InternalServerError().body("Failed to write file"));
        }
    }

    Ok(HttpResponse::Ok().body("Upload successful"))
}
