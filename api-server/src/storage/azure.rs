use futures::future::join_all;
use std::env;

async fn upload(filename: &str, base64_image: &str, format: &str) -> String {
    let mut builder = opendal::services::Azblob::default();
    let azblob_endpoint = env::var("AZBLOB_ENDPOINT").unwrap();
    let azblob_key = env::var("AZBLOB_KEY").unwrap();
    let azblob_container = env::var("AZBLOB_CONTAINER").unwrap();
    let azblob_account = env::var("AZBLOB_ACCOUNT").unwrap();
    builder.root("/");
    builder.container(&azblob_container);
    builder.endpoint(&azblob_endpoint);
    builder.account_name(&azblob_account);
    builder.account_key(&azblob_key);

    let op = opendal::Operator::new(builder).unwrap().finish();

    let output_image = data_encoding::BASE64.decode(base64_image.as_bytes()).unwrap();
    let content_type = format!("image/{}", format);
    match op.write_with(&filename, output_image).content_type(&content_type).await {
        Ok(_) => {
            format!("{}{}/{}", &azblob_endpoint, &azblob_container, &filename)
        },
        Err(e) => {
            tracing::error!("Azure upload error: {}", e);
            return String::from("");
        }
    }
}

pub async fn upload_images(images: &Vec<(String, &String)>, format: &str) -> Vec<String> {
    let image_urls = images
        .iter()
        .enumerate()
        .map(|(_i, (filename, base64_image))| {
            upload(filename, base64_image, format)
        })
        .collect::<Vec<_>>();
    join_all(image_urls).await
}
