use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpServer};
use env_logger::Env;
use errors::AppError;
use listenfd::ListenFd;
use std::sync::{Arc, Mutex};

#[allow(dead_code)]
mod config;
#[allow(dead_code)]
mod errors;
#[allow(dead_code)]
mod handlers;
#[allow(dead_code)]
mod middlewares;
#[allow(dead_code)]
mod models;

use crate::middlewares::CheckAuth;
use config::Config;
use handlers::configure;

#[derive(Debug)]
pub struct AppData {
    pub user_id: Option<String>,
}

impl AppData {
    fn new() -> Self {
        AppData { user_id: None }
    }
}

#[allow(unused_must_use)]
#[actix_rt::main]
async fn main() -> Result<(), AppError> {
    let mut listener = ListenFd::from_env();
    env_logger::from_env(Env::default().default_filter_or("info")).init();
    let config = Config::from_env();

    let db = config.get_db().await?;
    let app_data = web::Data::new(Arc::new(Mutex::new(AppData::new())));

    let mut server = HttpServer::new(move || {
        App::new()
            .wrap(Cors::default())
            .app_data(app_data.clone())
            .wrap(CheckAuth)
            .wrap(middleware::Logger::new("%a %r %s %Ts"))
            .data(db.clone())
            .configure(configure)
    });

    server = if let Some(i) = listener.take_tcp_listener(0).unwrap() {
        server.listen(i).unwrap()
    } else {
        server
            .bind(format!("{}:{}", config.host, config.port))
            .unwrap()
    };
    server.run().await;

    Ok(())
}
