mod image_proc;
mod scryfall;
use magicvision::cache::paths::get_data_dir;
use std::{path::PathBuf, time::SystemTime};
use tokio::fs;

use crate::image_proc::update_cache_with_jobs;
use crate::scryfall::{download_bulk_data, get_bulk_data_endpoint, update_images};
use magicvision::{
    cache::card_cache::{load_card_cache, save_card_cache},
    cache::paths::atomic_write,
    types::DynResult,
};

const DEFAULT_CARDS_URL: &str = "https://api.scryfall.com/bulk-data/default_cards";

#[tokio::main]
async fn main() -> DynResult<()> {
    // Check if bulk data endpoint has updated
    let current_url = get_bulk_data_endpoint(DEFAULT_CARDS_URL).await?;
    let stored_url = read_stored_url().await?;
    println!("\n           Latest API URL: {}", current_url);
    if let Ok(Some(url)) = read_stored_url().await {
        println!("Previously downloaded URL: {}\n", url);
    }

    let need_new_download: bool = match stored_url {
        Some(saved) => saved != current_url,
        None => true,
    };
    let json_filepath: PathBuf = match need_new_download {
        true => {
            println!("New bulk data avialable!");
            let fp = download_bulk_data(&current_url).await?;
            write_stored_url(&current_url).await?;
            fp
        }
        false => {
            println!("No new bulk data.");
            let filename = current_url
                .rfind('/')
                .map(|idx| &current_url[idx + 1..]) // Slice from after the last slash to the end
                .unwrap_or("data.ndjson");
            let fp = get_data_dir().join(filename).with_extension("ndjson");
            println!("Checking local file: {}", fp.to_string_lossy());
            fp
        }
    };

    println!("Updating image cache...");
    let jobs = update_images(&json_filepath).await?;
    println!("Done updating image cache.");

    if need_new_download {
        println!("Cleaning up old Scryfall files...");
        clean_old_raw_data_files().await?;
        println!("Done.")
    }

    println!("Loading hash cache...");
    let mut cache = load_card_cache()?;
    println!("Hash cache loaded.");

    println!("Updating hash cache with new images...");
    update_cache_with_jobs(&mut cache, &jobs)?;
    save_card_cache(&cache).await?;
    println!("Hash cache updated!");
    Ok(())
}

async fn read_stored_url() -> DynResult<Option<String>> {
    let path = get_data_dir().join("last_downloaded_url.txt");
    if !path.exists() {
        return Ok(None);
    }

    let url = fs::read_to_string(path).await?.trim().to_string();

    if url.is_empty() {
        Ok(None)
    } else {
        Ok(Some(url))
    }
}

async fn write_stored_url(url: &str) -> DynResult<()> {
    let path = get_data_dir().join("last_downloaded_url.txt");
    atomic_write(&path, url.as_bytes()).await?;
    Ok(())
}

async fn clean_old_raw_data_files() -> DynResult<()> {
    let mut entries = fs::read_dir(get_data_dir()).await?;
    let mut target_files: Vec<(PathBuf, SystemTime)> = Vec::new();

    // Collect valid .ndjson files' path and modified time
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();

        if path.extension().is_some_and(|ext| ext == "ndjson") {
            // Get metadata
            if let Ok(meta) = entry.metadata().await
                && let Ok(modified_time) = meta.modified()
            {
                target_files.push((path, modified_time));
            }
        }
    }

    target_files.sort_by_key(|(_, time)| *time);

    if !target_files.is_empty() {
        for (path, _) in target_files.iter().take(target_files.len() - 1) {
            fs::remove_file(path).await?;
        }
    }

    Ok(())
}
