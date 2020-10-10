use bson::{doc, oid::ObjectId};
use futures::StreamExt;
use mongodb::{Collection, Database};
use serde::{Deserialize, Serialize};

use crate::errors::{AppError, AppErrorType};

#[derive(Serialize, Deserialize, Debug)]
pub struct BlogPost {
    #[serde(rename = "_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub title: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    pub username: String
}

fn get_coll(db: &Database) -> Collection {
    db.collection("blog_posts")
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PostBlog {
    pub title: String,
    pub content: String,
}

impl BlogPost {
    pub fn new(title: String, content: String, user_id: &str, username: String) -> Self {
        BlogPost {
            id: None,
            title,
            user_id: Some(user_id.to_string()),
            content,
            username
        }
    }

    pub async fn save(&self, db: &Database) -> Result<(), AppError> {
        let coll = get_coll(db);
        match coll
            .insert_one(
                bson::to_document(&self).unwrap(),
                // doc! {
                //     "title": &self.title,
                //     "content": &self.content,
                //     "user_id": &self.user_id.as_ref().unwrap()
                // },
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

    pub async fn get_all_posts(db: &Database) -> Result<Vec<BlogPost>, AppError> {
        let coll = get_coll(&db);
        let mut cur = match coll.find(doc! {}, None).await {
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
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Comments {
    #[serde(rename = "_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: ObjectId,
    pub content: String,
    pub blog_id: ObjectId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replies: Option<Vec<Comments>>,
}

#[derive(Serialize, Debug, Deserialize)]
pub struct Replies{
    #[serde (rename= "_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub  id: Option<ObjectId>,
    pub user_id: ObjectId,
    pub content: String
}


#[derive(Deserialize, Debug)]
pub struct PostReply {
    pub user_id: String,
    pub content: String,
}

#[derive(Deserialize, Debug)]
pub struct PostComment {
    pub user_id: String,
    pub content: String,
    pub blog_id: String,
}


impl PostComment {
    pub async fn save(&self, db: &Database) -> Result<(), AppError> {
        let coll = db.collection("comments");
        match coll
            .insert_one(
                doc! {
                    "user_id": ObjectId::with_string(&self.user_id).unwrap()  ,
                    "blog_id": ObjectId::with_string(&self.blog_id).unwrap(),
                    "content": &self.content,
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

impl Comments {
    pub fn new(user_id: String, content: String, blog_id: String) -> Self {
        Comments {
            id: None,
            user_id: ObjectId::with_string(user_id.as_str()).unwrap(),
            blog_id: ObjectId::with_string(blog_id.as_str()).unwrap(),
            content,
            replies: None,
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

    pub async fn get_comments_by_id(db: &Database, comment_id: &str) -> Result<Comments, AppError>{
        let coll = db.collection("comments");
        let comment_id = match ObjectId::with_string(comment_id){
            Ok(val) => Ok(val),
            Err(_e) => Err(AppError{cause: Some(_e.to_string()), message: None, error_type: AppErrorType::InavlidId})
        }?;
        let res = match coll.find_one(doc!{"_id": comment_id }, None).await{
            Ok(doc) => Ok(doc),
            Err(_e) => Err(AppError{cause: Some(_e.to_string()), message: None, error_type: AppErrorType::InavlidId})
        }?;

        if res.is_none(){
            return   Err(AppError{cause: None, message: Some("Comment Not Found".to_string()), error_type: AppErrorType::NotFoundError});
        }

        Ok(bson::from_document::<Comments>(res.unwrap()).unwrap())
    }

    pub async fn save_reply(&self, db: &Database, reply: PostReply) -> Result<(), AppError> {

        let coll = db.collection("comments");

        let reply: Replies = Replies{
            id: None,
            user_id: ObjectId::with_string(reply.user_id.as_str()).unwrap(),
            content: reply.content,
        };

        match coll.update_one(doc!{
            "_id": &self.id.as_ref().unwrap()
        }, doc! {
            "$push": {
                "replies": bson::to_bson(&reply).unwrap() 
            }
        }, None).await{
            Ok(val) => Ok(val),
            Err(_e) =>  Err(AppError{cause: Some(_e.to_string()), message: None, error_type: AppErrorType::DatabaseError})
        }?;

        Ok(())
    }

}
