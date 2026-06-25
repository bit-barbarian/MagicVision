use futures::stream::{self, StreamExt};
use governor::{
    Quota, RateLimiter,
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
};
use reqwest::{
    Client,
    header::{ACCEPT, HeaderMap, HeaderValue, USER_AGENT},
};
use serde::Deserialize;
use std::{
    num::NonZeroU32,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use tokio::{
    fs::{self, File},
    io::{AsyncBufReadExt, BufReader},
};

use crate::{DATA_DIR, DynResult, atomic_write};

const MAX_RETRIES: usize = 3;
const MAX_CONCURRENT: usize = 20;
const RATE_LIMIT: NonZeroU32 = NonZeroU32::new(20).unwrap(); // Requests per second

#[derive(Debug, Deserialize)]
struct BulkDataMetaItem {
    download_uri: String,
}

#[derive(Debug, Deserialize)]
struct Card {
    id: String,
    name: String,
    set: String,
    collector_number: String,
    image_uris: Option<ImageUris>,
    card_faces: Option<Vec<CardFace>>,
}

#[derive(Debug, Deserialize)]
struct CardFace {
    image_uris: Option<ImageUris>,
}

#[derive(Debug, Deserialize)]
struct ImageUris {
    border_crop: String,
    normal: String,
}

pub struct Job {
    pub id: String,
    pub name: String,
    pub set: String,
    pub number: String,
    pub uris: Vec<(usize, String, String)>, // card_face, border_crop url, normal url
}
impl Job {
    pub fn image_path(&self, image_dir: &Path, face: &usize) -> PathBuf {
        image_dir.join(format!("{}_{}.jpg", self.id, face))
    }
}

fn scryfall_headers() -> DynResult<HeaderMap> {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_str("MagicVision/0.1.0")?);
    headers.insert(ACCEPT, HeaderValue::from_str("application/json")?);
    Ok(headers)
}

pub async fn get_bulk_data_endpoint(url: &str) -> DynResult<String> {
    let client = Client::new();
    let headers = scryfall_headers()?;

    // Fetch data & check status
    println!("Fetching latest bulk data endpoint...");
    let response = client.get(url).headers(headers).send().await?;
    if !response.status().is_success() {
        return Err(format!("Request failed: {}", response.status()).into());
    }

    // Deserialize into struct
    let bulk_data: BulkDataMetaItem = response.json().await?;
    Ok(bulk_data.download_uri)
}

pub async fn download_bulk_data(url: &str) -> DynResult<PathBuf> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()?;
    let headers = scryfall_headers()?;

    // Fetch data & check status
    println!("Fetching data... (this may take a while, ~500MB)");
    let response = client.get(url).headers(headers).send().await?;
    if !response.status().is_success() {
        return Err(format!("Failed fetching data: {}", response.status()).into());
    }
    let content = response.text().await?;
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
        .unwrap_or("data.ndjson");
    let file_path = Path::new(DATA_DIR).join(filename).with_extension("ndjson");
    println!("Saving data to: {}", &file_path.to_string_lossy());
    atomic_write(&file_path, processed_content.as_bytes()).await?;
    Ok(file_path)
}

pub async fn update_images(json_filepath: &Path) -> DynResult<Vec<Job>> {
    // Setup IO
    let image_dir = Path::new(DATA_DIR).join("images/");
    fs::create_dir_all(&image_dir).await?;
    let file = File::open(json_filepath).await?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    let client = Client::builder().timeout(Duration::from_secs(30)).build()?;
    let headers = scryfall_headers()?;
    let limiter = Arc::new(RateLimiter::direct(Quota::per_second(RATE_LIMIT)));

    let mut jobs: Vec<Job> = Vec::new();
    while let Some(line) = lines.next_line().await? {
        if line.trim().is_empty() {
            continue;
        }

        let Ok(card) = serde_json::from_str::<Card>(&line) else {
            continue;
        };

        let Some(uris) = get_image_uris(&card.image_uris, &card.card_faces) else {
            continue;
        };

        jobs.push(Job {
            id: card.id,
            name: card.name,
            set: card.set,
            number: card.collector_number,
            uris,
        });
    }

    stream::iter(&jobs)
        .for_each_concurrent(MAX_CONCURRENT, |job| {
            let client = client.clone();
            let headers = headers.clone();
            let limiter = limiter.clone();
            let image_dir = image_dir.clone();

            async move {
                process_job(job, client, headers, limiter, image_dir).await;
            }
        })
        .await;

    Ok(jobs)
}

async fn fetch_with_retry(
    client: &Client,
    primary_url: &str,
    fallback_url: &str,
    headers: &HeaderMap,
    max_retries: usize,
) -> DynResult<reqwest::Response> {
    // Use fallback on 404
    return match try_url(client, primary_url, headers, max_retries).await? {
        Some(r) => Ok(r),
        None => try_url(client, fallback_url, headers, max_retries)
            .await?
            .ok_or_else(|| "fallback failed after primary 404".into()),
    };

    async fn try_url(
        client: &Client,
        url: &str,
        headers: &HeaderMap,
        max_retries: usize,
    ) -> DynResult<Option<reqwest::Response>> {
        let mut attempt = 0;

        loop {
            attempt += 1;

            match client.get(url).headers(headers.clone()).send().await {
                Ok(r) => {
                    if r.status().is_success() {
                        return Ok(Some(r));
                    }
                    let status = r.status();

                    // Signal to try fallback on 404 (border_crop doesn't exist)
                    if status.as_u16() == 404 {
                        return Ok(None);
                    }
                    // retry on 429 (rate limit) / 5xx
                    if status.as_u16() == 429 || status.is_server_error() {
                        if attempt >= max_retries {
                            return Err(format!("failed after retries: {}", status).into());
                        }
                    } else {
                        return Err(format!("bad status: {}", status).into());
                    }
                }
                Err(e) => {
                    if attempt >= max_retries {
                        return Err(format!("http request error: {}", e).into());
                    }
                }
            }

            // exponential backoff + jitter
            let base = 100_u64 * 2_u64.pow(attempt as u32);
            let jitter = fastrand::u64(..100);
            tokio::time::sleep(Duration::from_millis(base + jitter)).await;
        }
    }
}

async fn process_job(
    job: &Job,
    client: Client,
    headers: HeaderMap,
    limiter: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
    image_dir: PathBuf,
) {
    let uris = &job.uris;
    for (face, uri, fallback_uri) in uris {
        let path = job.image_path(&image_dir, face);
        if path.try_exists().unwrap_or(false) {
            continue;
        }

        limiter.until_ready().await;
        println!(
            "Attempting to download: {} ({} {}), {} face",
            job.name,
            job.set,
            job.number,
            match face {
                0 => "front",
                1 => "back",
                _ => "other",
            }
        );

        let response =
            match fetch_with_retry(&client, uri, fallback_uri, &headers, MAX_RETRIES).await {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("request error: {e}");
                    return;
                }
            };
        let bytes = match response.bytes().await {
            Ok(b) => b,
            Err(e) => {
                eprintln!("read error: {e}");
                return;
            }
        };
        if let Err(e) = atomic_write(&path, &bytes).await {
            eprintln!("write error: {e}");
        }
    }
}

// Returns vector of (face_number, uri)
fn get_image_uris(
    image_uris: &Option<ImageUris>,
    card_faces: &Option<Vec<CardFace>>,
) -> Option<Vec<(usize, String, String)>> {
    // Check if card is single-faced
    if let Some(uris) = image_uris {
        let primary = uris.border_crop.clone();
        let fallback = uris.normal.clone();
        Some(vec![(0, primary, fallback)])

    // Check if card is multi-faced
    } else if let Some(faces) = card_faces {
        let mut face_uris: Vec<(usize, String, String)> = Vec::new();

        for (i, face) in faces.iter().enumerate() {
            if let Some(uris) = &face.image_uris {
                let primary = uris.border_crop.clone();
                let fallback = uris.normal.clone();
                face_uris.push((i, primary, fallback));
            }
        }
        Some(face_uris)

    // Fallback to no image at all
    } else {
        None
    }
}
