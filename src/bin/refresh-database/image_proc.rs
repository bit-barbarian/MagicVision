use image_hasher::Hasher;
use rayon::prelude::*;
use std::path::Path;
use uuid::Uuid;

use crate::scryfall::Job;
use magicvision::{
    cache::{
        card_cache::{CachedCard, CachedFace, CardCache},
        matching::get_hasher,
        paths::get_image_dir,
    },
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
    let image_dir = get_image_dir();

    let faces: Vec<CachedFace> = job
        .face_details
        .iter()
        .map(|face| {
            let image_path = job.image_path(&image_dir, &face.face_number);
            let oracle_text: String = match &face.oracle_text {
                Some(t) => t.to_owned(),
                None => String::from(""),
            };
            CachedFace::new(face.face_number, &image_path, hasher, oracle_text)
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
