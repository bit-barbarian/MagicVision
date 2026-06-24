mod scryfall;
use std::{fs, io::Write, path::Path};
use tokio::fs as tokio_fs;

use scryfall::{download_bulk_data, get_bulk_data_endpoint, update_images_async};

type DynError = Box<dyn std::error::Error + Send + Sync>;
type DynResult<T> = Result<T, DynError>;

const DEFAULT_CARDS_URL: &str = "https://api.scryfall.com/bulk-data/default_cards";
const DATA_DIR: &str = "/path/to/data/dir";

#[tokio::main]
async fn main() -> DynResult<()> {
    // Check if bulk data endpoint has updated
    let current_url = get_bulk_data_endpoint(DEFAULT_CARDS_URL).await?;
    let stored_url = read_stored_url()?;
    println!("\n           Latest API URL: {}", current_url);
    if let Ok(Some(url)) = read_stored_url() {
        println!("Previously downloaded URL: {}\n", url);
    }

    let need_new_download: bool = match stored_url {
        Some(saved) => saved != current_url,
        None => true,
    };
    let json_filepath = match need_new_download {
        true => {
            println!("New bulk data avialable!");
            write_stored_url(&current_url)?;
            download_bulk_data(&current_url).await?
        }
        false => {
            println!("No new bulk data.  Checking existing data.");
            let filename = current_url
                .rfind('/')
                .map(|idx| &current_url[idx + 1..]) // Slice from after the last slash to the end
                .unwrap_or("data.ndjson");
            format!("{}{}", DATA_DIR, filename)
        }
    };

    println!("Updating images...");
    update_images_async(&json_filepath).await?;

    Ok(())
}

fn read_stored_url() -> DynResult<Option<String>> {
    let path = Path::new(DATA_DIR).join("last_downloaded_url.txt");
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

fn write_stored_url(url: &str) -> DynResult<()> {
    let path = Path::new(DATA_DIR).join("last_downloaded_url.txt");
    fs::create_dir_all(DATA_DIR)?;

    let mut file = fs::File::create(path)?;
    writeln!(file, "{}", url)?;

    Ok(())
}

async fn atomic_write(path: &Path, bytes: &[u8]) -> DynResult<()> {
    let tmp_path = path.with_extension("tmp");

    tokio_fs::write(&tmp_path, bytes).await?;
    tokio_fs::rename(tmp_path, path).await?;
    Ok(())
}
