use crate::{
    config::crypto::CryptoService,
    errors::{AppError, AppErrorType},
};

use bson;
use futures::StreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId},
    Collection, Database,
};
use serde::{Deserialize, Serialize};

fn get_coll(db: &Database) -> Collection {
    db.collection("users")
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserCreds {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PatchUser {
    pub email: String,
    pub username: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    #[serde(rename = "_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub username: String,
    pub email: String,
    // #[serde(skip_serializing)]
    pub password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_avatar: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserDetails {
    #[serde(rename = "_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_avatar: Option<String>,
}

impl User {
    pub async fn save(&mut self, db: &Database) -> Result<(), AppError> {
        let coll = get_coll(&db);
        self.password = CryptoService::hash_password(self.password.clone()).await?;
        self.user_avatar = Some(format!(
            "https://test-blog-static.s3.ap-south-1.amazonaws.com{}",
            &self.user_avatar.as_ref().unwrap()
        ));

        match coll
            .insert_one(bson::to_document(self).unwrap(), None)
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

    pub async fn check_username(&self, db: &Database) -> Result<(), AppError> {
        let coll = get_coll(&db);
        match coll
            .find_one(
                doc! {
                    "username": self.username.as_str()
                },
                None,
            )
            .await
        {
            Ok(val) => {
                if !val.is_none() {
                    return Err(AppError {
                        cause: Some("USERNAME_EXISTS".to_string()),
                        message: None,
                        error_type: AppErrorType::ALREADYEXIST,
                    });
                }
                return Ok(());
            }
            Err(_e) => Err(AppError {
                cause: Some(_e.to_string()),
                message: None,
                error_type: AppErrorType::DatabaseError,
            }),
        }
    }

    pub async fn check_email(&self, db: &Database) -> Result<(), AppError> {
        let coll = get_coll(&db);

        match coll
            .find_one(
                doc! {
                    "email": self.email.as_str()
                },
                None,
            )
            .await
        {
            Ok(val) => {
                if !val.is_none() {
                    return Err(AppError {
                        cause: Some("EMAIL_EXISTS".to_string()),
                        message: None,
                        error_type: AppErrorType::ALREADYEXIST,
                    });
                }
                return Ok(());
            }
            Err(_e) => Err(AppError {
                cause: Some(_e.to_string()),
                message: None,
                error_type: AppErrorType::DatabaseError,
            }),
        }
    }

    pub async fn get_user_by_id(db: &Database, uid: &str) -> Result<UserDetails, AppError> {
        match ObjectId::with_string(uid) {
            Ok(id) => {
                let coll = get_coll(&db);
                match coll
                    .find_one(
                        doc! {
                            "_id": id
                        },
                        None,
                    )
                    .await
                {
                    Ok(user) => {
                        if user.is_none() {
                            return Err(AppError {
                                message: Some("No User Found".to_string()),
                                cause: None,
                                error_type: AppErrorType::NotFoundError,
                            });
                        }
                        let user = bson::from_document::<UserDetails>(user.unwrap()).unwrap();
                        Ok(user)
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

    pub async fn get_user_by_email(db: &Database, email: &str) -> Result<User, AppError> {
        let coll = get_coll(&db);

        match coll
            .find_one(
                doc! {
                    "email": email
                },
                None,
            )
            .await
        {
            Ok(user) => {
                if user.is_none() {
                    return Err(AppError {
                        cause: None,
                        message: Some("No User Found".to_string()),
                        error_type: AppErrorType::NotFoundError,
                    });
                }
                Ok(bson::from_document(user.unwrap()).unwrap())
            }
            Err(_e) => Err(AppError {
                cause: Some(_e.to_string()),
                message: None,
                error_type: AppErrorType::DatabaseError,
            }),
        }
    }

    pub async fn get_users(db: &Database) -> Result<Vec<User>, AppError> {
        let coll = get_coll(&db);
        let mut cursor = match coll.find(None, None).await {
            Ok(cur) => Ok(cur),
            Err(_e) => Err(AppError {
                cause: Some(_e.to_string()),
                message: None,
                error_type: AppErrorType::DatabaseError,
            }),
        }?;

        let mut res: Vec<User> = vec![];

        while let Some(val) = cursor.next().await {
            let doc: User = bson::from_document(val.unwrap()).unwrap();
            res.push(doc);
        }
        Ok(res)
    }

    pub async fn change_password(
        db: &Database,
        user_id: &str,
        password: &str,
    ) -> Result<(), AppError> {
        let coll = get_coll(&db);
        match coll
            .update_one(
                doc! {
                    "_id": convert_obj_id(user_id).await?
                },
                doc! {
                    "$set": {
                        "password": CryptoService::hash_password(password.to_string()).await?,
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

impl UserCreds {
    pub async fn validate(&self, db_data: &User) -> Result<bool, AppError> {
        return Ok(
            CryptoService::verify_hash(db_data.password.clone(), self.password.clone()).await?,
        );
    }
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

impl PatchUser {
    pub async fn patch_user_details(&self, db: &Database, user_id: &str) -> Result<(), AppError> {
        let coll = get_coll(&db);

        match coll
            .update_one(
                doc! {
                    "_id": convert_obj_id(user_id).await?
                },
                doc! {
                    "$set": {
                        "username": &self.username,
                        "email": &self.email,
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
