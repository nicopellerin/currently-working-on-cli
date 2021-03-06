use chrono::prelude::*;
use colored::*;
use dotenv;
use mongodb::{bson::doc, sync::Client};
use reqwest::blocking::multipart;
use serde::{Deserialize, Serialize};
use std::io;
use std::print;

#[derive(Serialize, Deserialize, Debug)]
struct MediaUrl {
    secure_url: String,
    width: i16,
    height: i16,
}

#[derive(Serialize, Deserialize, Debug)]
struct Task {
    text: String,
    media_url: String,
    width: i16,
    height: i16,
    created_at: String,
}

fn main() {
    dotenv::dotenv().ok();

    let mongo_uri = dotenv::var("MONGO_URI").unwrap();
    let mongo_db = dotenv::var("MONGO_DB").unwrap();
    let mongo_collection = dotenv::var("MONGO_COLLECTION").unwrap();

    let client = Client::with_uri_str(mongo_uri).unwrap();
    let database = client.database(&mongo_db);
    let collection = database.collection::<Task>(&mongo_collection);

    let mut input = String::new();
    let mut media_path: String = String::new();

    let date_now = Utc::now();

    // Input: "What are you currently working on?"
    print!("\x1B[2J\x1B[1;1H");
    println!(
        "{}",
        "What are you currently working on?"
            .blue()
            .bold()
            .to_string()
    );

    match io::stdin().read_line(&mut input) {
        Ok(input) => input,
        Err(error) => panic!("Invalid text: {}", error),
    };

    let parsed_input: String = input.trim().parse().unwrap();

    // Input: Image/video path
    println!("{}", "Upload media [path]".blue().bold().to_string());

    match io::stdin().read_line(&mut media_path) {
        Ok(media_path) => media_path,
        Err(error) => panic!("Invalid image path: {}", error),
    };

    let parsed_media_path: String = media_path.trim().replace("'", "").parse().unwrap();

    let parsed = match upload_image_cloudinary(parsed_media_path) {
        Ok(media_uploaded) => media_uploaded,
        Err(error) => panic!("Error uploading to Cloudinary: {}", error),
    };

    println!(
        "{}",
        "Media uploaded successfully!".green().bold().to_string()
    );

    let media_url = parsed.secure_url;
    let width = parsed.width;
    let height = parsed.height;

    let data = Task {
        text: parsed_input,
        media_url,
        width,
        height,
        created_at: date_now.to_string(),
    };

    collection.insert_one(data, None).unwrap();

    println!("{}", "Added to DB successfully!".green().bold().to_string());
}

fn upload_image_cloudinary(media_path: String) -> Result<MediaUrl, reqwest::Error> {
    let client = reqwest::blocking::Client::new();

    let cloudinary_id = dotenv::var("CLOUDINARY_ID").unwrap();
    let cloudinary_api_key = dotenv::var("CLOUDINARY_API_KEY").unwrap();
    let cloudinary_preset = dotenv::var("CLOUDINARY_PRESET").unwrap();

    let form = match multipart::Form::new()
        .text("api_key", cloudinary_api_key)
        .text("upload_preset", cloudinary_preset)
        .file("file", &media_path)
    {
        Ok(form) => form,
        Err(error) => panic!("Invalid form: {}", error),
    };

    let mut media_type = String::from("image");

    if media_path.contains(".mp4") {
        media_type = String::from("video")
    }

    let res = match client
        .post(format!(
            "https://api.cloudinary.com/v1_1/{}/{}/upload",
            cloudinary_id, media_type
        ))
        .multipart(form)
        .send()
    {
        Ok(res) => res,
        Err(err) => panic!("Error upload_image_cloudinary: {}", err),
    };

    let parsed_res = serde_json::from_str(&res.text().unwrap());

    Ok(parsed_res.unwrap())
}
