use image_hasher::{HashAlg, Hasher, HasherConfig};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::{collections::HashMap, path::PathBuf};
use tokio::fs;

use crate::scryfall::Job;
use crate::{DATA_DIR, DynResult, atomic_write};

pub type CardCache = HashMap<String, CachedCard>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedCard {
    pub id: String,
    pub name: String,
    pub set: String,
    pub number: String,
    pub faces: Vec<CachedFace>,
}
impl CachedCard {
    pub fn new(
        id: String,
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
    pub face: usize,
    pub image_path: PathBuf,
    pub phash: [u8; 32],
}
impl CachedFace {
    pub fn new(face: usize, image_path: &Path, hasher: &Hasher) -> DynResult<Self> {
        Ok(Self {
            face,
            image_path: image_path.into(),
            phash: Self::compute_phash(image_path, hasher)?,
        })
    }

    fn compute_phash(path: &Path, hasher: &Hasher) -> DynResult<[u8; 32]> {
        let image = image::open(path)?;
        let hash = hasher.hash_image(&image);
        Ok(hash.as_bytes().try_into()?)
    }
}

pub async fn load_card_cache() -> DynResult<CardCache> {
    let path = Path::new(DATA_DIR).join("card_cache.json");
    if path.exists() {
        println!("Hash cache found!");
        let file = fs::read(path).await?;
        Ok(serde_json::from_slice(&file)?)
    } else {
        println!("No cache found.  Creating new...");
        let new_cache: CardCache = HashMap::new();
        Ok(new_cache)
    }
}

pub async fn save_card_cache(cache: &CardCache) -> DynResult<()> {
    let path = Path::new(DATA_DIR).join("card_cache.json");
    let json = serde_json::to_vec(&cache)?;
    atomic_write(&path, &json).await
}

pub fn update_cache_with_jobs(cache: &mut CardCache, jobs: &[Job]) -> DynResult<()> {
    println!("Finding missing hashes...");
    let missing_jobs: Vec<&Job> = jobs
        .iter()
        .filter(|job| !cache.contains_key(&job.id))
        .collect();

    println!("Hashing new images...");
    let new_cards: Vec<(String, CachedCard)> = missing_jobs
        .par_iter()
        .filter_map(|job| {
            build_cached_card(job)
                .ok()
                .map(|card| (card.id.clone(), card))
        })
        .collect();

    println!("Adding {} new cards to cache...", new_cards.len());
    cache.extend(new_cards);
    Ok(())
}

fn build_cached_card(job: &Job) -> DynResult<CachedCard> {
    let hasher = HasherConfig::new()
        .hash_alg(HashAlg::Gradient)
        .hash_size(16, 16)
        .to_hasher();

    let image_dir = Path::new(DATA_DIR).join("images/");

    let faces: Vec<CachedFace> = job
        .uris
        .iter()
        .map(|(face_num, _, _)| {
            let image_path = job.image_path(&image_dir, face_num);
            CachedFace::new(*face_num, &image_path, &hasher)
        })
        .collect::<DynResult<_>>()?;

    Ok(CachedCard::new(
        job.id.clone(),
        job.name.clone(),
        job.set.clone(),
        job.number.clone(),
        faces,
    ))
}
