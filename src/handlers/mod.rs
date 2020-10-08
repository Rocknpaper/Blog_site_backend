use actix_web::web;

pub mod auth_handler;
pub mod blogpost_handler;
pub mod user_handler;

use self::auth_handler::post_login;
use self::blogpost_handler::{get_comment, get_post, get_posts, post_comments, post_posts, post_reply};
use self::user_handler::{get_user, get_users, post_user};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(post_login)
        .service(get_user)
        .service(post_user)
        .service(get_users)
        .service(get_post)
        .service(get_posts)
        .service(post_posts)
        .service(post_comments)
        .service(get_comment)
        .service(post_reply);
}
