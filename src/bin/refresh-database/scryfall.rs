use reqwest::{
    blocking,
    header::{ACCEPT, HeaderMap, HeaderValue, USER_AGENT},
};
use serde::Deserialize;
use std::{
    error::Error,
    fs::{self, File},
    io::{self, BufReader, Write},
    path::Path,
    thread::sleep,
    time::Duration,
};

#[derive(Debug, Deserialize)]
struct BulkDataMetaItem {
    download_uri: String,
}

#[derive(Debug, Deserialize)]
pub struct Card {
    pub id: String,
    pub name: String,
    pub set: String,
    pub collector_number: String,
    pub image_uris: Option<ImageUris>,
    pub phash: Option<u64>,
    pub card_faces: Option<Vec<CardFace>>,
}

#[derive(Debug, Deserialize)]
pub struct CardFace {
    pub image_uris: Option<ImageUris>,
}

#[derive(Debug, Deserialize)]
pub struct ImageUris {
    pub border_crop: String,
}

fn scryfall_headers() -> Result<HeaderMap, Box<dyn Error>> {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_str("MagicVision/0.1.0")?);
    headers.insert(ACCEPT, HeaderValue::from_str("application/json")?);
    Ok(headers)
}

pub fn get_bulk_data_endpoint(url: &str) -> Result<String, Box<dyn Error>> {
    let client = blocking::Client::new();
    let headers = scryfall_headers()?;

    // Fetch data & check status
    println!("Fetching latest bulk data endpoint...");
    let response = client.get(url).headers(headers).send()?;
    if !response.status().is_success() {
        return Err(format!("Request failed: {}", response.status()).into());
    }

    // Deserialize into struct
    let bulk_data: BulkDataMetaItem = response.json()?;
    Ok(bulk_data.download_uri)
}

pub fn download_bulk_data(url: &str, data_dir: &str) -> Result<String, Box<dyn Error>> {
    let client = blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()?;
    let headers = scryfall_headers()?;

    // Fetch data & check status
    println!("Fetching data... (this may take a while, ~500MB)");
    let response = client.get(url).headers(headers).send()?;
    if !response.status().is_success() {
        return Err(format!("Failed fetching data: {}", response.status()).into());
    }
    let content = response.text()?;
    println!("Download complete.");

    // Format to NDJSON
    println!("Cropping first and last lines...");
    let mut lines: Vec<&str> = content.lines().collect();
    if lines.len() > 2 {
        lines = lines[1..lines.len() - 1].to_vec();
    } else if !lines.is_empty() {
        lines = vec![];
    }

    println!("Removing trailing commas...");
    let processed: Vec<&str> = lines
        .into_iter()
        .map(|line| line.trim_end_matches(','))
        .collect();
    let processed_content = processed.join("\n");

    // Create the file
    let filename = url
        .rfind('/')
        .map(|idx| &url[idx + 1..]) // Slice from after the last slash to the end
        .unwrap_or("data.json");
    let file_path = Path::new(data_dir).join(filename);
    println!("Saving data to: {}{}", data_dir, filename);
    let mut file = fs::File::create(&file_path)?;
    file.write_all(processed_content.as_bytes())?;

    Ok(format!("{}{}", data_dir, filename))
}

pub fn read_stored_url(data_dir: &str) -> Result<Option<String>, Box<dyn Error>> {
    let path = Path::new(data_dir).join("last_downloaded_url.txt");
    if !path.exists() {
        return Ok(None);
    }

    let url = fs::read_to_string(path)?.trim().to_string();

    if url.is_empty() {
        Ok(None)
    } else {
        Ok(Some(url))
    }
}

pub fn write_stored_url(url: &str, data_dir: &str) -> Result<(), Box<dyn Error>> {
    let path = Path::new(data_dir).join("last_downloaded_url.txt");
    fs::create_dir_all(data_dir)?;

    let mut file = fs::File::create(path)?;
    writeln!(file, "{}", url)?;

    Ok(())
}

pub fn update_images(json_filepath: &str, data_dir: &str) -> Result<(), Box<dyn Error>> {
    // Setup IO
    let image_dir = Path::new(data_dir).join("images/border_crop");
    fs::create_dir_all(&image_dir)?;
    let input_file = File::open(json_filepath)?;
    let reader = BufReader::new(input_file);
    let card_deserializer = serde_json::Deserializer::from_reader(reader);

    // Setup api client
    let client = blocking::Client::new();
    let headers = scryfall_headers()?;

    let mut card_count: u32 = 0;
    let mut download_count: u32 = 0;
    let mut no_image_count: u32 = 0;
    let mut cache_count: u32 = 0;
    let mut error_count: u32 = 0;

    for result in card_deserializer.into_iter::<Card>() {
        if card_count >= 20 {
            break;
        }
        card_count += 1;
        let card = match result {
            Ok(card) => card,
            Err(e) => {
                println!("Error reading card from json: {}", e);
                error_count += 1;
                continue;
            }
        };

        // Check if front image already exists in cache
        let check_filename = format!("{}_0.jpg", card.id);
        let file_path = Path::new(&image_dir).join(&check_filename);
        if file_path.try_exists()? {
            println!(
                "Exists in cache: {} ({} {})",
                &card.name, &card.set, &card.collector_number
            );
            cache_count += 1;
            continue;
        }

        // Find URIs and download front (and back if exists) image(s)
        if let Some(uris) = get_image_uris(&card) {
            for uri in uris {
                println!(
                    "Downloading: {} ({} {}), face: {}",
                    &card.name,
                    &card.set,
                    &card.collector_number,
                    match uri.0 {
                        0 => "front",
                        1 => "back",
                        _ => "other",
                    }
                );
                let response = client.get(uri.1).headers(headers.clone()).send()?;
                if !response.status().is_success() {
                    return Err(format!("Failed getting image: {}", response.status()).into());
                }
                sleep(Duration::from_millis(50));
                download_count += 1;

                // Save to fs
                let filename = format!("{}_{}.jpg", card.id, uri.0);
                let file_path = Path::new(&image_dir).join(&filename);
                let mut file = fs::File::create(&file_path)?;
                io::copy(&mut response.bytes()?.as_ref(), &mut file)?;
            }
        } else {
            println!(
                "No image for: {} ({} {})",
                &card.name, &card.set, &card.collector_number,
            );
            no_image_count += 1;
        }
    }

    println!("Total cards checked: {}", card_count);
    println!("Total cards in cache: {}", cache_count);
    println!("Total cards downloaded: {}", download_count);
    println!("Total cards with no image: {}", no_image_count);
    println!("Total cards with error: {}", error_count);
    Ok(())
}

// Returns vector of (face_number, uri)
fn get_image_uris(card: &Card) -> Option<Vec<(usize, &str)>> {
    // Check if card is single-faced
    if let Some(uris) = &card.image_uris {
        Some(vec![(0, &uris.border_crop)])

    // Check if card is multi-faced
    } else if let Some(faces) = &card.card_faces {
        let mut face_uris: Vec<(usize, &str)> = Vec::new();

        for (i, face) in faces.iter().enumerate() {
            if let Some(uris) = &face.image_uris {
                face_uris.push((i, &uris.border_crop));
            }
        }
        Some(face_uris)

    // Fallback to no image at all
    } else {
        None
    }
}
