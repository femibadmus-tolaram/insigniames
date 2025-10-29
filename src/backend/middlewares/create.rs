use actix_web::{dev::ServiceRequest, dev::ServiceResponse, Error, HttpResponse};
use actix_web::dev::{Transform, Service};
use futures::future::{LocalBoxFuture, Ready, ready};
use crate::backend::models::Permission;
use r2d2_sqlite::SqliteConnectionManager;
use r2d2::Pool;
use std::task::{Context, Poll};
use actix_session::SessionExt;
use actix_web::body::BoxBody;
// use actix_web::http::StatusCode;

pub struct CheckCreate {
    pub model: &'static str,
    pub conn_data: actix_web::web::Data<Pool<SqliteConnectionManager>>,
}

impl<S> Transform<S, ServiceRequest> for CheckCreate
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Transform = CheckCreateMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(CheckCreateMiddleware {
            service,
            model: self.model,
            conn_data: self.conn_data.clone(),
        }))
    }
}

pub struct CheckCreateMiddleware<S> {
    service: S,
    model: &'static str,
    conn_data: actix_web::web::Data<Pool<SqliteConnectionManager>>,
}

impl<S> Service<ServiceRequest> for CheckCreateMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let conn_data = self.conn_data.clone();
        let model = self.model;
        let user_id: Option<i32> = req.get_session().get("user_id").unwrap_or(None);

        if user_id.is_none() {
            let res = HttpResponse::Found()
                .append_header(("Location", "/auth/signin"))
                .finish();
            return Box::pin(async move { Ok(req.into_response(res.map_into_boxed_body())) });
        }

        let uid = user_id.unwrap();
        let conn = conn_data.get().unwrap();

        let perm_opt = Permission::all_for_user(&conn, uid)
            .unwrap()
            .into_iter()
            .find(|p| p.model == model);

        if let Some(p) = &perm_opt {
            if p.can_create {
                let fut = self.service.call(req);
                return Box::pin(async move { fut.await });
            }
        }

        // let body = match perm_opt {
        //     Some(p) => format!("User {} has no CREATE permission on {}, found permission for model {}", uid, model, p.model),
        //     None => format!("User {} has no CREATE permission on {}, no permission found", uid, model),
        // };
        // let res = HttpResponse::new(StatusCode::FORBIDDEN).set_body(BoxBody::new(body));
        let res = HttpResponse::Forbidden().json(&serde_json::json!({ "message": "Permission Denied" }));
        Box::pin(async move { Ok(req.into_response(res)) })
    }
}
