use std::sync::{Arc, Mutex};

use actix_web::{get, post, web, HttpResponse};
use mongodb::Database;
use serde_json::json;

use crate::{
    errors::AppError,
    models::{ user::User, blogs::{BlogPost, Comments, PostBlog, PostComment, PostReply}},
    AppData,
};

#[get("/blog/{id}")]
pub async fn get_post(
    id: web::Path<String>,
    db: web::Data<Database>,
) -> Result<HttpResponse, AppError> {
    let res = BlogPost::get_post_by_id(db.get_ref(), id.as_str()).await?;
    Ok(HttpResponse::Ok().json(res))
}

#[get("/blogs")]
pub async fn get_posts(db: web::Data<Database>) -> Result<HttpResponse, AppError> {
    let val = BlogPost::get_all_posts(db.get_ref()).await?;
    Ok(HttpResponse::Ok().json(val))
}

#[post("/blog")]
pub async fn post_posts(
    db: web::Data<Database>,
    data: web::Json<PostBlog>,
    app_data: web::Data<Arc<Mutex<AppData>>>,
) -> Result<HttpResponse, AppError> {

    let user_id = app_data.lock().unwrap().user_id.as_ref().unwrap().clone();

    let user = User::get_user_by_id(db.get_ref(), user_id.as_str()).await?;

    let blog = BlogPost::new(
        data.title.to_owned(),
        data.content.to_owned(),
        user_id.as_str(),
        user.username
    );
    blog.save(db.get_ref()).await?;
    Ok(HttpResponse::Ok().body(json!({
        "Status": "OK",
        "response": 200
    })))
}

#[post("/comment")]
pub async fn post_comments(
    db: web::Data<Database>,
    form_data: web::Json<PostComment>,
) -> Result<HttpResponse, AppError> {
    form_data.save(db.get_ref()).await?;
    Ok(HttpResponse::Ok().json(json! ({
        "Status": "Ok",
        "response": 200
    })))
}

#[get("/comment/{id}")]
pub async fn get_comment(
    db: web::Data<Database>,
    id: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    println!("{}", id.to_string());
    let res = Comments::get_comments_by_post(db.get_ref(), id.to_string().as_str()).await?;
    Ok(HttpResponse::Ok().json(res))
}

#[post("/reply/{id}")]
pub async fn post_reply(db: web::Data<Database>, id: web::Path<String>, data: web::Json<PostReply>) -> Result<HttpResponse, AppError>{
    let comment = Comments::get_comments_by_id(db.get_ref(), id.as_str()).await?;
    comment.save_reply(db.get_ref(), data.0).await?;
    Ok(HttpResponse::Ok().json(json!({
        "Status": "OK",
        "response": 200
    })))

}