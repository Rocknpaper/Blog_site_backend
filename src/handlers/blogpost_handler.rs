use std::sync::{Arc, Mutex};

use actix_web::{delete, get, patch, post, web, HttpResponse};
use mongodb::Database;
use serde_json::json;

use crate::{
    errors::AppError,
    models::{
        blogs::{BlogPost, Comments, IncOrDec, PostBlog, PostComment, PostReply, Replies},
        user::User,
    },
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

#[get("/blog/user/{user_id}")]
pub async fn get_blog_by_uid(db: web::Data<Database>, user_id: web::Path<String>) -> Result<HttpResponse, AppError>{
    let blogs = BlogPost::get_posts_by_uid(db.get_ref(), user_id.as_str()).await?;
    Ok(HttpResponse::Ok().json(blogs))
}

#[patch("/blog/{blog_id}")]
pub async fn patch_posts(
    db: web::Data<Database>,
    blog: web::Json<PostBlog>,
    blog_id: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    blog.patch_posts(db.get_ref(), blog_id.as_str()).await?;
    Ok(HttpResponse::Ok().json(json! ({
        "Status": "OK",
        "response": 200
    })))
}

#[delete("/blog/{blog_id}")]
pub async fn delete_blog(
    db: web::Data<Database>,
    blog_id: web::Path<String>,
    app_data: web::Data<Arc<Mutex<AppData>>>,
) -> Result<HttpResponse, AppError> {
    let user_id = app_data.lock().unwrap().user_id.as_ref().unwrap().clone();

    BlogPost::delete_blog(db.get_ref(), blog_id.as_str(), user_id.as_str()).await?;

    Ok(HttpResponse::Ok().json(json!({
        "Status": "OK",
        "response": 200
    })))
}

#[get("/user-blog")]
pub async fn get_user_posts(
    db: web::Data<Database>,
    app_data: web::Data<Arc<Mutex<AppData>>>,
) -> Result<HttpResponse, AppError> {
    let user_id = app_data.lock().unwrap().user_id.as_ref().unwrap().clone();

    let blog = BlogPost::get_posts_by_uid(db.get_ref(), user_id.as_str()).await?;

    Ok(HttpResponse::Ok().json(blog))
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
        user_id,
        user.username,
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
    let id = form_data.save(db.get_ref()).await?;
    Ok(HttpResponse::Ok().json(json! ({
        "Status": "Ok",
        "response": 200,
        "id": id
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

#[patch("/comment/{id}")]
pub async fn patch_comment(
    db: web::Data<Database>,
    id: web::Path<String>,
    data: web::Json<PostComment>,
) -> Result<HttpResponse, AppError> {
    data.patch_comments(db.get_ref(), id.as_str()).await?;

    Ok(HttpResponse::Ok().json(json! ({
        "Status": "Ok",
        "response": 200,
    })))
}

#[delete("/comment/{id}")]
pub async fn delete_comment(
    db: web::Data<Database>,
    id: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    Comments::delete(db.get_ref(), id.as_str()).await?;
    Ok(HttpResponse::Ok().json(json!({
        "Status": "OK",
        "response": 200,
    })))
}

#[post("/reply-comment/{id}")]
pub async fn post_reply(
    db: web::Data<Database>,
    id: web::Path<String>,
    data: web::Json<PostReply>,
) -> Result<HttpResponse, AppError> {
    let comment = Comments::get_comments_by_id(db.get_ref(), id.as_str()).await?;
    let id = comment.save_reply(db.get_ref(), data.0).await?;
    Ok(HttpResponse::Ok().json(json!({
        "Status": "OK",
        "response": 200,
        "id": id
    })))
}

#[patch("/reply-comment/{comment_id}/{reply_id}")]
pub async fn patch_reply(
    db: web::Data<Database>,
    params: web::Path<(String, String)>,
    data: web::Json<PostReply>,
) -> Result<HttpResponse, AppError> {
    let (comment_id, reply_id) = params.into_inner();

    data.patch_replies(db.get_ref(), comment_id.as_str(), reply_id.as_str())
        .await?;

    Ok(HttpResponse::Ok().json(json!({
        "Status": "OK",
        "response": 200,
    })))
}

#[delete("/reply-comment/{comment_id}/{reply_id}")]
pub async fn delete_reply(
    db: web::Data<Database>,
    params: web::Path<(String, String)>,
) -> Result<HttpResponse, AppError> {
    let (comment_id, reply_id) = params.into_inner();

    Comments::delete_reply(db.get_ref(), comment_id.as_str(), reply_id.as_str()).await?;

    Ok(HttpResponse::Ok().json(json!({
        "Status": "OK",
        "response": 200,
    })))
}

#[patch("/upvote/inc/{blog_id}")]
pub async fn upvote_handler_inc(
    db: web::Data<Database>,
    blog_id: web::Path<String>,
    app_data: web::Data<Arc<Mutex<AppData>>>,
) -> Result<HttpResponse, AppError> {
    let user_id = app_data.lock().unwrap().user_id.as_ref().unwrap().clone();

    BlogPost::upvote(
        db.get_ref(),
        blog_id.as_str(),
        user_id.as_str(),
        IncOrDec::INC,
    )
    .await?;

    Ok(HttpResponse::Ok().json(json!({
        "Status": "OK",
        "response": 200
    })))
}

#[patch("/upvote/dec/{blog_id}")]
pub async fn upvote_handler_dec(
    db: web::Data<Database>,
    blog_id: web::Path<String>,
    app_data: web::Data<Arc<Mutex<AppData>>>,
) -> Result<HttpResponse, AppError> {
    let user_id = app_data.lock().unwrap().user_id.as_ref().unwrap().clone();

    BlogPost::upvote(
        db.get_ref(),
        blog_id.as_str(),
        user_id.as_str(),
        IncOrDec::DEC,
    )
    .await?;

    Ok(HttpResponse::Ok().json(json!({
        "Status": "OK",
        "response": 200
    })))
}

#[patch("/downvote/inc/{blog_id}")]
pub async fn downvote_handler_inc(
    db: web::Data<Database>,
    blog_id: web::Path<String>,
    app_data: web::Data<Arc<Mutex<AppData>>>,
) -> Result<HttpResponse, AppError> {
    let user_id = app_data.lock().unwrap().user_id.as_ref().unwrap().clone();

    BlogPost::downvote(
        db.get_ref(),
        blog_id.as_str(),
        user_id.as_str(),
        IncOrDec::INC,
    )
    .await?;

    Ok(HttpResponse::Ok().json(json!({
        "Status": "OK",
        "response": 200
    })))
}

#[patch("/downvote/dec/{blog_id}")]
pub async fn downvote_handler_dec(
    db: web::Data<Database>,
    blog_id: web::Path<String>,
    app_data: web::Data<Arc<Mutex<AppData>>>,
) -> Result<HttpResponse, AppError> {
    let user_id = app_data.lock().unwrap().user_id.as_ref().unwrap().clone();

    BlogPost::downvote(
        db.get_ref(),
        blog_id.as_str(),
        user_id.as_str(),
        IncOrDec::DEC,
    )
    .await?;

    Ok(HttpResponse::Ok().json(json!({
        "Status": "OK",
        "response": 200
    })))
}

#[patch("/like/inc/{comment_id}")]
pub async fn like_handler_inc(
    db: web::Data<Database>,
    comment_id: web::Path<String>,
    app_data: web::Data<Arc<Mutex<AppData>>>,
) -> Result<HttpResponse, AppError> {
    let user_id = app_data.lock().unwrap().user_id.as_ref().unwrap().clone();

    Comments::like(
        db.get_ref(),
        comment_id.as_str(),
        user_id.as_str(),
        IncOrDec::INC,
    )
    .await?;

    Ok(HttpResponse::Ok().json(json!({
        "Status": "OK",
        "response": 200
    })))
}

#[patch("/like/dec/{comment_id}")]
pub async fn like_handler_dec(
    db: web::Data<Database>,
    comment_id: web::Path<String>,
    app_data: web::Data<Arc<Mutex<AppData>>>,
) -> Result<HttpResponse, AppError> {
    let user_id = app_data.lock().unwrap().user_id.as_ref().unwrap().clone();

    Comments::like(
        db.get_ref(),
        comment_id.as_str(),
        user_id.as_str(),
        IncOrDec::DEC,
    )
    .await?;

    Ok(HttpResponse::Ok().json(json!({
        "Status": "OK",
        "response": 200
    })))
}

#[patch("/dislike/inc/{comment_id}")]
pub async fn dislike_handler_inc(
    db: web::Data<Database>,
    comment_id: web::Path<String>,
    app_data: web::Data<Arc<Mutex<AppData>>>,
) -> Result<HttpResponse, AppError> {
    let user_id = app_data.lock().unwrap().user_id.as_ref().unwrap().clone();

    Comments::dislike(
        db.get_ref(),
        comment_id.as_str(),
        user_id.as_str(),
        IncOrDec::INC,
    )
    .await?;

    Ok(HttpResponse::Ok().json(json!({
        "Status": "OK",
        "response": 200
    })))
}

#[patch("/dislike/dec/{comment_id}")]
pub async fn dislike_handler_dec(
    db: web::Data<Database>,
    comment_id: web::Path<String>,
    app_data: web::Data<Arc<Mutex<AppData>>>,
) -> Result<HttpResponse, AppError> {
    let user_id = app_data.lock().unwrap().user_id.as_ref().unwrap().clone();

    Comments::dislike(
        db.get_ref(),
        comment_id.as_str(),
        user_id.as_str(),
        IncOrDec::DEC,
    )
    .await?;

    Ok(HttpResponse::Ok().json(json!({
        "Status": "OK",
        "response": 200
    })))
}

#[patch("/reply/like/inc/{comment_id}/{reply_id}")]
pub async fn reply_like_inc(
    db: web::Data<Database>,
    params: web::Path<(String, String)>,
    app_data: web::Data<Arc<Mutex<AppData>>>,
) -> Result<HttpResponse, AppError> {
    let user_id = app_data.lock().unwrap().user_id.as_ref().unwrap().clone();

    let (comment_id, reply_id) = params.into_inner();

    Replies::likes(
        db.get_ref(),
        user_id.as_str(),
        comment_id.as_str(),
        reply_id.as_str(),
        IncOrDec::INC,
    )
    .await?;

    Ok(HttpResponse::Ok().json(json!({
        "Status": "OK",
        "response": 200
    })))
}

#[patch("/reply/like/dec/{comment_id}/{reply_id}")]
pub async fn reply_like_dec(
    db: web::Data<Database>,
    params: web::Path<(String, String)>,
    app_data: web::Data<Arc<Mutex<AppData>>>,
) -> Result<HttpResponse, AppError> {
    let user_id = app_data.lock().unwrap().user_id.as_ref().unwrap().clone();

    let (comment_id, reply_id) = params.into_inner();

    Replies::likes(
        db.get_ref(),
        user_id.as_str(),
        comment_id.as_str(),
        reply_id.as_str(),
        IncOrDec::DEC,
    )
    .await?;

    Ok(HttpResponse::Ok().json(json!({
        "Status": "OK",
        "response": 200
    })))
}

#[patch("/reply/dislike/inc/{comment_id}/{reply_id}")]
pub async fn reply_dislike_inc(
    db: web::Data<Database>,
    params: web::Path<(String, String)>,
    app_data: web::Data<Arc<Mutex<AppData>>>,
) -> Result<HttpResponse, AppError> {
    let user_id = app_data.lock().unwrap().user_id.as_ref().unwrap().clone();

    let (comment_id, reply_id) = params.into_inner();

    Replies::dislikes(
        db.get_ref(),
        user_id.as_str(),
        comment_id.as_str(),
        reply_id.as_str(),
        IncOrDec::INC,
    )
    .await?;

    Ok(HttpResponse::Ok().json(json!({
        "Status": "OK",
        "response": 200
    })))
}

#[patch("/reply/dislike/dec/{comment_id}/{reply_id}")]
pub async fn reply_dislike_dec(
    db: web::Data<Database>,
    params: web::Path<(String, String)>,
    app_data: web::Data<Arc<Mutex<AppData>>>,
) -> Result<HttpResponse, AppError> {
    let user_id = app_data.lock().unwrap().user_id.as_ref().unwrap().clone();

    let (comment_id, reply_id) = params.into_inner();

    Replies::dislikes(
        db.get_ref(),
        user_id.as_str(),
        comment_id.as_str(),
        reply_id.as_str(),
        IncOrDec::DEC,
    )
    .await?;

    Ok(HttpResponse::Ok().json(json!({
        "Status": "OK",
        "response": 200
    })))
}
