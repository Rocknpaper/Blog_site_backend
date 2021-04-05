use lettre_email::EmailBuilder;
use lettre::{Transport, SmtpClient};
use lettre::smtp::authentication::IntoCredentials;
use std::env::var;

use crate::errors::{AppError, AppErrorType};

pub struct Emailer{
    pub email: String,
    pub password: String
}

impl Emailer{

    pub fn from_defaults() -> Emailer{
        Emailer{
            email: var("email").unwrap(),
            password: var("password").unwrap()
        }
    }

    pub async fn new_service(&self, to: String, content: i32) -> Result<(), AppError> {
        
        let mail = EmailBuilder::new()
                                    .to(to)
                                    .from(&*self.email)
                                    .subject("Password Recovery")
                                    .html(format!("<p>You Have Requested to reset Password</p>
                                        <h3>{}</h3> 
                                        <p>is your recovery code</p>
                                    ", content))
                                    .build().unwrap();




        let creds = (&*self.email, &*self.password).into_credentials();

        let mut mailer = SmtpClient::new_simple("smtp.gmail.com")
                                        .unwrap()
                                        .credentials(creds)
                                        .transport();

        match mailer.send(mail.into()){
            Ok(_) => Ok(()),
            Err(_e) => Err(AppError{cause: Some(_e.to_string()), message: None, error_type: AppErrorType::EmailError})
        }
    }
}
