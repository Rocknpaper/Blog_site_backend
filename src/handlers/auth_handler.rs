use actix_web::{post, web, HttpResponse};
use mongodb::Database;
use serde_json::json;

use crate::models::user::{User, UserCreds};
use crate::{config::jwt::Claims, errors::AppError};

#[post("/auth/user")]
pub async fn post_login(
    db: web::Data<Database>,
    form_data: web::Json<UserCreds>,
) -> Result<HttpResponse, AppError> {
    let user = User::get_user_by_email(db.get_ref(), form_data.email.as_str()).await?;
    if !form_data.validate(&user).await? {
        return Ok(HttpResponse::Unauthorized().body("Incorrect Password"));
    }
    let jwt = Claims::encode_req(user.id.as_ref().unwrap().to_string().as_str()).await?;
    Ok(HttpResponse::Ok().json(json!({"_id": user.id, "username": user.username, "email": user.email, "user_avatar": user.user_avatar ,"jwt": jwt })))
}
