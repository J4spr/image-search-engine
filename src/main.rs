use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::Value;
use std::fs::File;
use std::io::{self, copy};
use std::path::Path;

#[derive(Deserialize)]
struct UnsplashPhoto {
    id: String,
    urls: Urls,
}

#[derive(Deserialize)]
struct Urls {
    regular: String,
}
fn main() {
    // dotenv init
    let _ = dotenv_vault::dotenv();

    let key = std::env::var("API_KEY").unwrap_or("".to_string());

    // get input from the user for the query
    let mut query = String::new();

    println!("What photos do you wanna download?");
    io::stdin()
        .read_line(&mut query)
        .expect("Failed to read input");

    let mut pages = String::new();

    println!("How many pages do you want to see?");
    io::stdin()
        .read_line(&mut pages)
        .expect("Failed to read input");

    let pages_as_int: u8 = pages
        .trim()
        .parse()
        .expect("Failed to parse pages to integer");

    let response = send_request(&key, query.trim().to_string(), pages_as_int);

    let download_dir = "./images/";
    std::fs::create_dir_all(download_dir).expect("Failed to create directory");
    for photo in response {
        download_images(&photo.urls.regular, &photo.id, download_dir);
    }
}
fn send_request(key: &str, query: String, per_page: u8) -> Vec<UnsplashPhoto> {
    // create a new reqwest client
    let client: Client = Client::new();

    // get the api url
    let url: &str = "https://api.unsplash.com/search/photos";

    // make a GET request to the api url
    let response = client
        .get(url)
        .query(&[("query", query), ("per_page", per_page.to_string())])
        .header("Authorization", format!("Client-ID {}", key))
        .send()
        .unwrap();

    // check if the status is 200 (OK)
    if response.status().is_success() {
        // deserialize the json repsonse into a struct
        let json: Value = response.json().unwrap();

        // create a vector to store the photo information
        let mut photos: Vec<UnsplashPhoto> = Vec::new();

        // exctract all data we need and print photo information
        for photo in json["results"].as_array().unwrap() {
            let unsplash_photo: UnsplashPhoto = serde_json::from_value(photo.clone()).unwrap();
            photos.push(unsplash_photo);
        }

        return photos;
    }

    Vec::new()
}

fn download_images(url: &str, filename: &str, download_dir: &str) {
    // Sanitize the filename to remove invalid characters
    let sanitized_filename = filename
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect::<String>();

    // Split the URL by the query parameters ('?') to get the path
    let path_parts: Vec<&str> = url.split('?').collect();
    let path = path_parts[0];

    // Extract the file extension from the path
    let extension = Path::new(path)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("jpg");

    // Construct the filepath using the sanitized filename and file extension
    let filepath = format!("{}/{}.{}", download_dir, sanitized_filename, extension);

    let mut response = reqwest::blocking::get(url).expect("Failed to send request");
    let mut file = File::create(filepath).expect("Failed to create file");

    copy(&mut response, &mut file).expect("Failed to download image");
}

