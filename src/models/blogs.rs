use bson::{doc, oid::ObjectId, DateTime};
use chrono::Utc;
use futures::StreamExt;
use mongodb::{options::FindOptions, Collection, Database};
use serde::{Deserialize, Serialize};

use crate::errors::{AppError, AppErrorType};

#[derive(Serialize, Deserialize, Debug)]
pub struct Votes {
    pub users: Vec<ObjectId>,
    pub count: i32,
}

impl Votes {
    pub fn new() -> Self {
        Votes {
            users: vec![],
            count: 0,
        }
    }
}

#[derive(PartialEq)]
pub enum IncOrDec {
    INC,
    DEC,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BlogPost {
    #[serde(rename = "_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub title: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    pub username: String,
    pub created_at: DateTime,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upvotes: Option<Votes>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub downvotes: Option<Votes>,
}

fn get_coll(db: &Database) -> Collection {
    db.collection("blog_posts")
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PostBlog {
    pub title: String,
    pub content: String,
}

impl PostBlog {
    pub async fn patch_posts(&self, db: &Database, blog_id: &str) -> Result<(), AppError> {
        let coll = get_coll(db);

        let blog_id = match ObjectId::with_string(blog_id) {
            Ok(val) => Ok(val),
            Err(_e) => Err(AppError {
                cause: Some(_e.to_string()),
                message: None,
                error_type: AppErrorType::InavlidId,
            }),
        }?;

        match coll
            .update_one(
                doc! {
                    "_id": blog_id
                },
                doc! {
                "$set": {
                    "title": self.title.as_str(),
                    "content": self.content.as_str()
                }
                },
                None,
            )
            .await
        {
            Ok(_) => Ok(()),
            Err(_e) => Err(AppError {
                message: None,
                cause: Some(_e.to_string()),
                error_type: AppErrorType::DatabaseError,
            }),
        }
    }
}

impl BlogPost {
    pub fn new(title: String, content: String, user_id: String, username: String) -> Self {
        BlogPost {
            id: None,
            title,
            user_id: Some(user_id.as_str().to_string()),
            content,
            username,
            created_at: DateTime(Utc::now()),
            upvotes: Some(Votes::new()),
            downvotes: Some(Votes::new()),
        }
    }

    pub async fn upvote(
        db: &Database,
        blog_id: &str,
        user_id: &str,
        patch_type: IncOrDec,
    ) -> Result<(), AppError> {
        let coll = get_coll(db);

        let blog_id = match ObjectId::with_string(blog_id) {
            Ok(val) => Ok(val),
            Err(_e) => Err(AppError {
                message: None,
                cause: Some(_e.to_string()),
                error_type: AppErrorType::InavlidId,
            }),
        }?;

        match coll
            .update_one(
                doc! {
                    "_id": blog_id
                },
                doc! {
                    if patch_type == IncOrDec::INC {"$push"} else {"$pull"}: {
                        "upvotes.users": ObjectId::with_string(user_id).unwrap()
                    },
                     "$inc": {
                        "upvotes.count": if patch_type == IncOrDec::DEC {-1} else {1}
                    }
                },
                None,
            )
            .await
        {
            Ok(_) => Ok(()),
            Err(_e) => Err(AppError {
                cause: Some(_e.to_string()),
                message: None,
                error_type: AppErrorType::DatabaseError,
            }),
        }
    }

    pub async fn downvote(
        db: &Database,
        blog_id: &str,
        user_id: &str,
        patch_type: IncOrDec,
    ) -> Result<(), AppError> {
        let coll = get_coll(db);

        let blog_id = match ObjectId::with_string(blog_id) {
            Ok(val) => Ok(val),
            Err(_e) => Err(AppError {
                message: None,
                cause: Some(_e.to_string()),
                error_type: AppErrorType::InavlidId,
            }),
        }?;

        match coll
            .update_one(
                doc! {
                    "_id": blog_id
                },
                doc! {
                    if patch_type == IncOrDec::INC {"$push"} else {"$pull"}: {
                        "downvotes.users": ObjectId::with_string(user_id).unwrap()
                    },
                     "$inc": {
                        "downvotes.count": if patch_type == IncOrDec::DEC {-1} else {1}
                    }
                },
                None,
            )
            .await
        {
            Ok(_) => Ok(()),
            Err(_e) => Err(AppError {
                cause: Some(_e.to_string()),
                message: None,
                error_type: AppErrorType::DatabaseError,
            }),
        }
    }

    pub async fn save(&self, db: &Database) -> Result<(), AppError> {
        let coll = get_coll(db);
        match coll
            .insert_one(bson::to_document(&self).unwrap(), None)
            .await
        {
            Ok(_) => Ok(()),
            Err(_e) => Err(AppError {
                cause: Some(_e.to_string()),
                message: None,
                error_type: AppErrorType::DatabaseError,
            }),
        }
    }

    pub async fn get_all_posts(db: &Database) -> Result<Vec<BlogPost>, AppError> {
        let coll = get_coll(&db);
        let options = FindOptions::builder()
            .sort(doc! {"upvotes.count": -1 })
            .build();
        let mut cur = match coll.find(doc! {}, options).await {
            Ok(cur) => Ok(cur),
            Err(_e) => Err(AppError {
                cause: Some(_e.to_string()),
                message: None,
                error_type: AppErrorType::DatabaseError,
            }),
        }?;
        let mut res: Vec<BlogPost> = vec![];
        while let Some(val) = cur.next().await {
            match val {
                Ok(doc) => {
                    res.push(bson::from_document::<BlogPost>(doc).unwrap());
                    Ok(())
                }
                Err(_e) => Err(AppError {
                    cause: Some(_e.to_string()),
                    message: None,
                    error_type: AppErrorType::DatabaseError,
                }),
            }?;
        }

        Ok(res)
    }

    pub async fn get_post_by_id(db: &Database, id: &str) -> Result<BlogPost, AppError> {
        match ObjectId::with_string(id) {
            Ok(id) => {
                let coll = get_coll(&db);
                match coll
                    .find_one(
                        doc! {
                            "_id": ObjectId::from(id)
                        },
                        None,
                    )
                    .await
                {
                    Ok(post) => {
                        if post.is_none() {
                            return Err(AppError {
                                cause: None,
                                message: Some("No Post Found".to_string()),
                                error_type: AppErrorType::NotFoundError,
                            });
                        }
                        let post: BlogPost = bson::from_document(post.unwrap()).unwrap();
                        Ok(post)
                    }
                    Err(_e) => Err(AppError {
                        cause: Some(_e.to_string()),
                        message: None,
                        error_type: AppErrorType::DatabaseError,
                    }),
                }
            }
            Err(_e) => {
                println!("{}", _e);
                Err(AppError {
                    cause: Some(_e.to_string()),
                    message: Some("Invalid Id".to_string()),
                    error_type: AppErrorType::InavlidId,
                })
            }
        }
    }

    pub async fn get_posts_by_uid(db: &Database, user_id: &str) -> Result<Vec<BlogPost>, AppError> {
        let coll = get_coll(&db);

        let mut cur = match coll.find(doc! {"user_id": user_id}, None).await {
            Ok(any) => Ok(any),
            Err(_e) => Err(AppError {
                cause: Some(_e.to_string()),
                message: None,
                error_type: AppErrorType::DatabaseError,
            }),
        }?;
        let mut res: Vec<BlogPost> = vec![];

        while let Some(doc) = cur.next().await {
            res.push(bson::from_document(doc.unwrap()).unwrap());
        }

        Ok(res)
    }

    pub async fn delete_blog(db: &Database, blog_id: &str, user_id: &str) -> Result<(), AppError> {
        let blog_id = match ObjectId::with_string(blog_id) {
            Ok(val) => Ok(val),
            Err(_e) => Err(AppError {
                cause: Some(_e.to_string()),
                message: None,
                error_type: AppErrorType::InavlidId,
            }),
        }?;

        let coll = get_coll(db);
        match coll
            .delete_one(
                doc! {
                    "_id": blog_id,
                    "user_id": user_id
                },
                None,
            )
            .await
        {
            Ok(_) => Ok(()),
            Err(_e) => Err(AppError {
                cause: Some(_e.to_string()),
                message: None,
                error_type: AppErrorType::DatabaseError,
            }),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Comments {
    #[serde(rename = "_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: ObjectId,
    pub username: String,
    pub content: String,
    pub blog_id: ObjectId,
    pub created_at: DateTime,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replies: Option<Vec<Replies>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub likes: Option<Votes>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dislikes: Option<Votes>,
}

impl Comments {
    pub async fn new(
        user_id: &str,
        username: &str,
        content: &str,
        blog_id: &str,
    ) -> Result<Self, AppError> {
        Ok(Comments {
            id: None,
            user_id: convert_obj_id(user_id).await?,
            blog_id: convert_obj_id(blog_id).await?,
            content: content.to_string(),
            username: username.to_string(),
            created_at: DateTime(Utc::now()),
            replies: None,
            likes: Some(Votes::new()),
            dislikes: Some(Votes::new()),
        })
    }
}

#[derive(Serialize, Debug, Deserialize)]
pub struct Replies {
    #[serde(rename = "_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: ObjectId,
    pub created_at: DateTime,
    pub username: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub likes: Option<Votes>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dislikes: Option<Votes>,
}

impl Replies {
    pub async fn new(user_id: &str, username: &str, content: &str) -> Result<Self, AppError> {
        Ok(Replies {
            id: Some(ObjectId::new()),
            user_id: convert_obj_id(user_id).await?,
            username: username.to_string(),
            content: content.to_string(),
            created_at: DateTime(Utc::now()),
            likes: Some(Votes::new()),
            dislikes: Some(Votes::new()),
        })
    }

    pub async fn likes(
        db: &Database,
        user_id: &str,
        comment_id: &str,
        reply_id: &str,
        patch_type: IncOrDec,
    ) -> Result<(), AppError> {
        let coll = db.collection("comments");

        match coll
            .update_one(
                doc! {
                    "_id": convert_obj_id(comment_id).await?,
                    "replies._id":
                        convert_obj_id(reply_id).await?,

                },
                doc! {
                    if patch_type == IncOrDec::INC {"$push"} else {"$pull"}: {
                        "replies.$.likes.users": convert_obj_id(user_id).await?
                    },
                     "$inc": {
                        "replies.$.likes.count": if patch_type == IncOrDec::DEC {-1} else {1}
                    }
                },
                None,
            )
            .await
        {
            Ok(_) => Ok(()),
            Err(_e) => Err(AppError {
                cause: Some(_e.to_string()),
                message: None,
                error_type: AppErrorType::DatabaseError,
            }),
        }
    }

    pub async fn dislikes(
        db: &Database,
        user_id: &str,
        comment_id: &str,
        reply_id: &str,
        patch_type: IncOrDec,
    ) -> Result<(), AppError> {
        let coll = db.collection("comments");

        match coll
            .update_one(
                doc! {
                    "_id": convert_obj_id(comment_id).await?,
                    "replies._id":
                        convert_obj_id(reply_id).await?,

                },
                doc! {
                    if patch_type == IncOrDec::INC {"$push"} else {"$pull"}: {
                        "replies.$.dislikes.users": convert_obj_id(user_id).await?
                    },
                     "$inc": {
                        "replies.$.dislikes.count": if patch_type == IncOrDec::DEC {-1} else {1}
                    }
                },
                None,
            )
            .await
        {
            Ok(_) => Ok(()),
            Err(_e) => Err(AppError {
                cause: Some(_e.to_string()),
                message: None,
                error_type: AppErrorType::DatabaseError,
            }),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct PostReply {
    pub user_id: String,
    pub username: String,
    pub content: String,
}

#[derive(Deserialize, Debug)]
pub struct PostComment {
    pub user_id: String,
    pub username: String,
    pub content: String,
    pub blog_id: String,
}

async fn convert_obj_id(id: &str) -> Result<ObjectId, AppError> {
    match ObjectId::with_string(id) {
        Ok(val) => Ok(val),
        Err(_e) => Err(AppError {
            cause: Some(_e.to_string()),
            message: None,
            error_type: AppErrorType::InavlidId,
        }),
    }
}

impl PostComment {
    pub async fn save(&self, db: &Database) -> Result<String, AppError> {
        let coll = db.collection("comments");

        let comment = Comments::new(
            &self.user_id.as_str(),
            &self.username.as_str(),
            &self.content.as_str(),
            &self.blog_id.as_str(),
        )
        .await?;

        match coll
            .insert_one(bson::to_document(&comment).unwrap(), None)
            .await
        {
            Ok(m) => {
                Ok(m.inserted_id.as_object_id().unwrap().to_hex())
            }
            Err(_e) => Err(AppError {
                cause: Some(_e.to_string()),
                message: None,
                error_type: AppErrorType::DatabaseError,
            }),
        }
    }
}

impl Comments {
    pub async fn like(
        db: &Database,
        comment_id: &str,
        user_id: &str,
        patch_type: IncOrDec,
    ) -> Result<(), AppError> {
        let coll = db.collection("comments");
        match coll
            .update_one(
                doc! {"_id": convert_obj_id(comment_id).await?},
                doc! {
                    if patch_type == IncOrDec::INC {"$push"} else {"$pull"}: {
                        "likes.users": ObjectId::with_string(user_id).unwrap()
                    },
                     "$inc": {
                        "likes.count": if patch_type == IncOrDec::DEC {-1} else {1}
                    }
                },
                None,
            )
            .await
        {
            Ok(_) => Ok(()),
            Err(_e) => Err(AppError {
                cause: Some(_e.to_string()),
                message: None,
                error_type: AppErrorType::DatabaseError,
            }),
        }
    }

    pub async fn dislike(
        db: &Database,
        comment_id: &str,
        user_id: &str,
        patch_type: IncOrDec,
    ) -> Result<(), AppError> {
        let coll = db.collection("comments");
        match coll
            .update_one(
                doc! {"_id": convert_obj_id(comment_id).await?},
                doc! {
                    if patch_type == IncOrDec::INC {"$push"} else {"$pull"}: {
                        "dislikes.users": ObjectId::with_string(user_id).unwrap()
                    },
                     "$inc": {
                        "dislikes.count": if patch_type == IncOrDec::DEC {-1} else {1}
                    }
                },
                None,
            )
            .await
        {
            Ok(_) => Ok(()),
            Err(_e) => Err(AppError {
                cause: Some(_e.to_string()),
                message: None,
                error_type: AppErrorType::DatabaseError,
            }),
        }
    }

    pub async fn get_comments_by_post(
        db: &Database,
        blog_id: &str,
    ) -> Result<Vec<Comments>, AppError> {
        let coll = db.collection("comments");
        let mut cur = match coll
            .find(
                doc! {
                    "blog_id": ObjectId::with_string(blog_id).unwrap()
                },
                None,
            )
            .await
        {
            Ok(val) => Ok(val),
            Err(_e) => Err(AppError {
                cause: Some(_e.to_string()),
                message: None,
                error_type: AppErrorType::DatabaseError,
            }),
        }?;

        let mut res: Vec<Comments> = vec![];

        while let Some(val) = cur.next().await {
            match val {
                Ok(doc) => {
                    res.push(bson::from_document::<Comments>(doc).unwrap());
                    Ok(())
                }
                Err(_e) => Err(AppError {
                    cause: Some(_e.to_string()),
                    message: None,
                    error_type: AppErrorType::DatabaseError,
                }),
            }?;
        }
        Ok(res)
    }

    pub async fn get_comments_by_id(db: &Database, comment_id: &str) -> Result<Comments, AppError> {
        let coll = db.collection("comments");
        let comment_id = match ObjectId::with_string(comment_id) {
            Ok(val) => Ok(val),
            Err(_e) => Err(AppError {
                cause: Some(_e.to_string()),
                message: None,
                error_type: AppErrorType::InavlidId,
            }),
        }?;
        let res = match coll.find_one(doc! {"_id": comment_id }, None).await {
            Ok(doc) => Ok(doc),
            Err(_e) => Err(AppError {
                cause: Some(_e.to_string()),
                message: None,
                error_type: AppErrorType::InavlidId,
            }),
        }?;

        if res.is_none() {
            return Err(AppError {
                cause: None,
                message: Some("Comment Not Found".to_string()),
                error_type: AppErrorType::NotFoundError,
            });
        }

        Ok(bson::from_document::<Comments>(res.unwrap()).unwrap())
    }

    pub async fn save_reply(&self, db: &Database, reply: PostReply) -> Result<String, AppError> {
        let coll = db.collection("comments");

        let reply = Replies::new(
            reply.user_id.as_str(),
            reply.username.as_str(),
            reply.content.as_str(),
        )
        .await?;

        match coll
            .update_one(
                doc! {
                    "_id": &self.id.as_ref().unwrap()
                },
                doc! {
                    "$push": {
                        "replies": bson::to_bson(&reply).unwrap()
                    }
                },
                None,
            )
            .await
        {
            Ok(val) => Ok(val),
            Err(_e) => Err(AppError {
                cause: Some(_e.to_string()),
                message: None,
                error_type: AppErrorType::DatabaseError,
            }),
        }?;

        Ok(reply.id.unwrap().to_string())
    }

    pub async fn delete(db: &Database, comment_id: &str) -> Result<(), AppError> {
        let coll = db.collection("comments");
        let comment_id = convert_obj_id(comment_id).await?;
        match coll
            .delete_one(
                doc! {
                    "_id": comment_id
                },
                None,
            )
            .await
        {
            Ok(_) => Ok(()),
            Err(_e) => Err(AppError {
                cause: Some(_e.to_string()),
                message: None,
                error_type: AppErrorType::DatabaseError,
            }),
        }
    }

    pub async fn delete_reply(
        db: &Database,
        comment_id: &str,
        reply_id: &str,
    ) -> Result<(), AppError> {
        let coll = db.collection("comments");

        let comment_id = convert_obj_id(comment_id).await?;
        let reply_id = convert_obj_id(reply_id).await?;

        match coll
            .update_one(
                doc! {
                    "_id": comment_id,
                },
                doc! {
                    "$pull": {
                        "replies": {
                            "_id":
                            reply_id
                        }
                    }
                },
                None,
            )
            .await
        {
            Ok(_) => Ok(()),
            Err(_e) => Err(AppError {
                cause: Some(_e.to_string()),
                message: None,
                error_type: AppErrorType::DatabaseError,
            }),
        }
    }
}
