use actix_web::web;

pub mod auth_handler;
pub mod blogpost_handler;
pub mod user_handler;

use self::auth_handler::post_login;
use self::blogpost_handler::{
    delete_blog, delete_comment, delete_reply, dislike_handler_dec, dislike_handler_inc,
    downvote_handler_dec, downvote_handler_inc, get_comment, get_post, get_posts, get_user_posts,
    like_handler_dec, like_handler_inc, patch_comment, patch_posts, patch_reply, post_comments,
    post_posts, post_reply, reply_dislike_dec, reply_dislike_inc, reply_like_dec, reply_like_inc,
    upvote_handler_dec, upvote_handler_inc,
};
use self::user_handler::{get_user, /*get_users,*/ post_user};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(post_login)
        .service(get_user)
        .service(post_user)
        //        .service(get_users)
        .service(get_post)
        .service(get_posts)
        .service(post_posts)
        .service(post_comments)
        .service(get_comment)
        .service(post_reply)
        .service(upvote_handler_inc)
        .service(upvote_handler_dec)
        .service(downvote_handler_inc)
        .service(downvote_handler_dec)
        .service(get_user_posts)
        .service(patch_posts)
        .service(delete_blog)
        .service(like_handler_inc)
        .service(like_handler_dec)
        .service(dislike_handler_inc)
        .service(dislike_handler_dec)
        .service(reply_like_inc)
        .service(reply_like_dec)
        .service(reply_dislike_inc)
        .service(reply_dislike_dec)
        .service(delete_comment)
        .service(patch_comment)
        .service(patch_reply)
        .service(delete_reply);
}
