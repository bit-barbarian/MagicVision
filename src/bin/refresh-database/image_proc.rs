use image_hasher::Hasher;
use rayon::prelude::*;
use std::path::Path;
use uuid::Uuid;

use crate::scryfall::Job;
use magicvision::{
    cache::{
        card_cache::{CachedCard, CachedFace, CardCache},
        matching::get_hasher,
    },
    constants::DATA_DIR,
    types::DynResult,
};

pub fn update_cache_with_jobs(cache: &mut CardCache, jobs: &[Job]) -> DynResult<()> {
    println!("Finding missing hashes...");
    let missing_jobs: Vec<&Job> = jobs
        .iter()
        .filter(|job| !cache.contains_key(&job.id))
        .collect();

    println!("Hashing new images...");
    let hasher = get_hasher();
    let new_cards: Vec<(Uuid, CachedCard)> = missing_jobs
        .par_iter()
        .filter_map(|job| {
            build_cached_card(job, &hasher)
                .ok()
                .map(|card| (card.id, card))
        })
        .collect();

    println!("Adding {} new cards to cache...", new_cards.len());
    cache.extend(new_cards);
    Ok(())
}

fn build_cached_card(job: &Job, hasher: &Hasher) -> DynResult<CachedCard> {
    let image_dir = Path::new(DATA_DIR).join("images/");

    let faces: Vec<CachedFace> = job
        .uris
        .iter()
        .map(|(face_num, _, _)| {
            let image_path = job.image_path(&image_dir, face_num);
            CachedFace::new(*face_num, &image_path, hasher)
        })
        .collect::<DynResult<_>>()?;

    Ok(CachedCard::new(
        job.id,
        job.name.clone(),
        job.set.clone(),
        job.number.clone(),
        faces,
    ))
}
