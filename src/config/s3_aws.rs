use actix_multipart::{Field, Multipart};
use actix_web::web;
use bytes::Bytes;
use dotenv::dotenv;
use futures::StreamExt;
use s3::{bucket::Bucket, creds::Credentials, region::Region};
use std::io::Write;

#[derive(Debug, Clone)]
pub struct Tmpfile {
    pub name: String,
    pub tmp_path: String,
}

impl Tmpfile {
    fn new(filename: &str) -> Tmpfile {
        Tmpfile {
            name: filename.to_string(),
            tmp_path: format!("./tmp/{}", filename),
        }
    }
}

pub async fn save_file(temp_files: Vec<Tmpfile>) -> std::io::Result<Vec<String>> {
    let mut res = Vec::new();

    for file in temp_files {
        match upload_file(get_s3_bucket().await, &file.tmp_path, &file.name).await {
            Ok(_) => {
                remove_file(&file.tmp_path);
                res.push(file.name);
            }
            Err(_) => println!("Upload Failed"),
        }
    }

    Ok(res)
}

pub async fn split_payload(payload: &mut Multipart) -> (Bytes, Vec<Tmpfile>) {
    let mut tmp_files = Vec::new();
    let mut data = Bytes::new();

    while let Some(item) = payload.next().await {
        let mut field: Field = item.unwrap();
        let content_type = field.content_disposition().unwrap();
        println!("{:?}", content_type);
        let name = content_type.get_name().unwrap();

        if name == "data" {
            while let Some(chuck) = field.next().await {
                data = chuck.unwrap()
            }
        } else {
            match content_type.get_filename() {
                Some(filename) => {
                    let temp_file = Tmpfile::new(filename);
                    let temp_path = temp_file.tmp_path.clone();
                    let mut f = web::block(move || std::fs::File::create(&temp_path))
                        .await
                        .unwrap();
                    while let Some(chunk) = field.next().await {
                        let data = chunk.unwrap();
                        f = web::block(move || f.write_all(&data).map(|_| f))
                            .await
                            .unwrap();
                    }
                    tmp_files.push(temp_file.clone());
                }
                None => println!("No File Found"),
            }
        }
    }
    (data, tmp_files)
}

pub async fn get_s3_bucket() -> Bucket {
    dotenv().ok();

    let creds = Credentials::from_env().unwrap();

    Bucket::new(
        &std::env::var("AWS_STORAGE_BUCKET_NAME").unwrap(),
        Region::ApSouth1,
        creds,
    )
    .unwrap()
}

pub async fn upload_file(buck: Bucket, file: &str, filename: &str) -> std::io::Result<String> {
    let content = async_std::fs::read(file).await.unwrap();

    let (_, code) = buck
        .put_object(format!("/{}", filename), &content)
        .await
        .unwrap();

    println!("{}", code);
    Ok(format!("/{}", filename))
}

pub fn remove_file(path: &str) {
    std::fs::remove_file(path).unwrap();
}
