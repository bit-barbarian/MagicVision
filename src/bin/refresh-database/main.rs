use std::error::Error;

mod scryfall;
use scryfall::{
    download_bulk_data, get_bulk_data_endpoint, read_stored_url, update_images, write_stored_url,
};

const DEFAULT_CARDS_URL: &str = "https://api.scryfall.com/bulk-data/default_cards";
const DATA_DIR: &str = "/path/to/data/dir";

fn main() -> Result<(), Box<dyn Error>> {
    // Check if bulk data endpoint has updated
    let current_url = get_bulk_data_endpoint(DEFAULT_CARDS_URL)?;
    let stored_url = read_stored_url(DATA_DIR)?;
    println!("\n           Latest API URL: {}", current_url);
    if let Ok(Some(url)) = read_stored_url(DATA_DIR) {
        println!("Previously downloaded URL: {}\n", url);
    }

    let need_new_download: bool = match stored_url {
        Some(saved) => saved != current_url,
        None => true,
    };
    let json_filepath = match need_new_download {
        true => {
            println!("New bulk data avialable!");
            write_stored_url(&current_url, DATA_DIR)?;
            download_bulk_data(&current_url, DATA_DIR)?
        }
        false => {
            println!("No new bulk data.  Checking existing data.");
            let filename = current_url
                .rfind('/')
                .map(|idx| &current_url[idx + 1..]) // Slice from after the last slash to the end
                .unwrap_or("data.json");
            format!("{}{}", DATA_DIR, filename)
        }
    };

    println!("Updating images...");
    update_images(&json_filepath, DATA_DIR)?;

    Ok(())
}
