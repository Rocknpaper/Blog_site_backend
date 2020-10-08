// use std::sync::Mutex;
// use std::sync::Arc;

use actix_multipart::Multipart;
use actix_web::{get, post, web, HttpResponse};
use mongodb::Database;
use serde_json::json;

use crate::models::user::User;
use crate::{config::s3_aws, errors::AppError, errors::AppErrorType};

#[post("/user")]
pub async fn post_user(
    db: web::Data<Database>,
    mut payload: Multipart,
) -> Result<HttpResponse, AppError> {
    let (data, file) = s3_aws::split_payload(&mut payload).await;

    let _bucket = s3_aws::get_s3_bucket().await;
    let mut user: User = serde_json::from_slice(&data).unwrap();

    let ext: Vec<&str> = file[0].name.split(".").collect();

    let filename = format!("{}.{}", user.username, ext[1]);

    match s3_aws::upload_file(_bucket, file[0].tmp_path.as_str(), filename.as_str()).await {
        Ok(link) => {
            s3_aws::remove_file(&file[0].tmp_path[..]);

            user.user_avatar = Some(link);
            user.save(db.get_ref()).await?;

            Ok(HttpResponse::Ok().json(json!({
                "response": 200,
                "Status": "Ok"
            })))
        }
        Err(_e) => Err(AppError {
            cause: Some(_e.to_string()),
            message: Some("Upload Failed".to_string()),
            error_type: AppErrorType::FileUploadError,
        }),
    }
}

#[get("/users")]
pub async fn get_users(db: web::Data<Database>) -> Result<HttpResponse, AppError> {
    let users = User::get_users(db.get_ref()).await?;
    Ok(HttpResponse::Ok().json(users))
}

#[get("/user/{uid}")]
pub async fn get_user(
    db: web::Data<Database>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let user = User::get_user_by_id(db.get_ref(), &path).await?;
    Ok(HttpResponse::Ok().json(user))
}
