use image_hasher::Hasher;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::Path, path::PathBuf};
use uuid::Uuid;

use crate::{
    cache::paths::{atomic_write, get_data_dir},
    types::DynResult,
};

pub type CardCache = HashMap<Uuid, CachedCard>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedCard {
    pub id: Uuid,
    pub name: String,
    pub set: String,
    pub number: String,
    pub faces: Vec<CachedFace>,
}
impl CachedCard {
    pub fn new(
        id: Uuid,
        name: String,
        set: String,
        number: String,
        faces: Vec<CachedFace>,
    ) -> Self {
        Self {
            id,
            name,
            set,
            number,
            faces,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedFace {
    pub face: u8,
    pub image_path: PathBuf,
    pub oracle_text: String,
    pub phash: [u8; 32],
}
impl CachedFace {
    pub fn new(
        face: u8,
        image_path: &Path,
        hasher: &Hasher,
        oracle_text: String,
    ) -> DynResult<Self> {
        Ok(Self {
            face,
            image_path: image_path.into(),
            oracle_text,
            phash: Self::compute_phash(image_path, hasher)?,
        })
    }

    fn compute_phash(path: &Path, hasher: &Hasher) -> DynResult<[u8; 32]> {
        let image = image::open(path)?;
        let hash = hasher.hash_image(&image);
        Ok(hash.as_bytes().try_into()?)
    }
}

pub fn load_card_cache() -> DynResult<CardCache> {
    let path = get_data_dir().join("cards.json");
    if path.exists() {
        let file = fs::read(path)?;
        Ok(serde_json::from_slice(&file)?)
    } else {
        println!("No cache found.  Creating new...");
        let new_cache: CardCache = HashMap::new();
        Ok(new_cache)
    }
}

pub async fn save_card_cache(cache: &CardCache) -> DynResult<()> {
    let path = get_data_dir().join("cards.json");
    let json = serde_json::to_vec(&cache)?;
    atomic_write(&path, &json).await
}
