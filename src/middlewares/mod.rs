use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};

use actix_service::{Service, Transform};
use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    web, Error,
};
use futures::future::{ok, Either, Ready};

use crate::{config::jwt::Claims, errors::AppError, errors::AppErrorType, AppData};

pub struct CheckAuth;

impl<S, B> Transform<S> for CheckAuth
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = CheckAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(CheckAuthMiddleware { service })
    }
}
pub struct CheckAuthMiddleware<S> {
    service: S,
}

impl<S, B> Service for CheckAuthMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Either<S::Future, Ready<Result<Self::Response, Self::Error>>>;

    fn poll_ready(&mut self, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        if req.method() == "OPTIONS" {
            return Either::Left(self.service.call(req));
        }
        let excep = vec![
            "auth/user",
            "/user",
            "/user/",
            "/blogs",
            "/blog/",
            "/comment/",
        ];

        for add in excep {
            if req.path().contains(add) {
                return Either::Left(self.service.call(req));
            }
        }

        let _auth = req.headers().get("authorization");

        match _auth {
            Some(_) => {
                let _split: Vec<&str> = _auth.unwrap().to_str().unwrap().split("Bearer").collect();
                let token = _split[1].trim();
                match Claims::decode_req(token) {
                    Ok(_token) => {
                        // let user = UserData{user_id: Some(_token.claims.sub)};
                        {
                            let app_data =
                                req.app_data::<web::Data<Arc<Mutex<AppData>>>>().unwrap();
                            let mut data = app_data.lock().unwrap();
                            data.user_id = Some(_token.claims.sub);
                        }
                        return Either::Left(self.service.call(req));
                    }
                    Err(_e) => {
                        println!("{:?}", _e);
                        Either::Right(ok(req.error_response(_e)))
                    }
                }
            }
            None => Either::Right(ok(req.error_response(AppError {
                cause: Some("No Jwt token Attached".to_string()),
                message: Some("Add the JWT token Header".to_string()),
                error_type: AppErrorType::JWtTokenError,
            }))),
        }
    }
}
