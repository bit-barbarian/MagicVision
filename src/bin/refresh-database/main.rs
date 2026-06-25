mod image_proc;
mod scryfall;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use tokio::{fs, io::AsyncWriteExt};

use crate::image_proc::{load_card_cache, save_card_cache, update_cache_with_jobs};
use crate::scryfall::{download_bulk_data, get_bulk_data_endpoint, update_images};

type DynError = Box<dyn std::error::Error + Send + Sync>;
type DynResult<T> = Result<T, DynError>;

const DEFAULT_CARDS_URL: &str = "https://api.scryfall.com/bulk-data/default_cards";
const DATA_DIR: &str = "/path/to/data/dir";

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
            let fp = PathBuf::from_str(DATA_DIR)?
                .join(filename)
                .with_extension("ndjson");
            println!("Checking local file: {}", fp.to_string_lossy());
            fp
        }
    };

    println!("Updating image cache...");
    let jobs = update_images(&json_filepath).await?;
    println!("Done updating image cache.");

    println!("Loading hash cache...");
    let mut cache = load_card_cache().await?;
    println!("Hash cache loaded.");

    println!("Updating hash cache with new images...");
    update_cache_with_jobs(&mut cache, &jobs)?;
    save_card_cache(&cache).await?;
    println!("Hash cache updated!");
    Ok(())
}

async fn read_stored_url() -> DynResult<Option<String>> {
    let path = Path::new(DATA_DIR).join("last_downloaded_url.txt");
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
    let path = Path::new(DATA_DIR).join("last_downloaded_url.txt");
    fs::create_dir_all(DATA_DIR).await?;

    atomic_write(&path, url.as_bytes()).await?;
    Ok(())
}

async fn atomic_write(path: &Path, bytes: &[u8]) -> DynResult<()> {
    let tmp_path = path.with_extension("tmp");
    let mut file = fs::File::create(&tmp_path).await?;

    file.write_all(bytes).await?;
    file.sync_all().await?;
    fs::rename(tmp_path, path).await?;
    Ok(())
}
