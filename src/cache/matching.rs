use arrayvec::ArrayVec;
use image_hasher::{HashAlg, Hasher, HasherConfig, ImageHash};
use rayon::prelude::*;
use std::cmp::Ordering;
use uuid::Uuid;

use crate::{cache::card_cache::CardCache, constants::NUM_MATCHES};

pub struct MatchEntry {
    pub hash: ImageHash,
    pub card_id: Uuid,
    pub face: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MatchResult {
    pub distance: u32,
    pub card_id: Uuid,
    pub face: u8,
}
impl Ord for MatchResult {
    fn cmp(&self, other: &Self) -> Ordering {
        self.distance.cmp(&other.distance)
    }
}

impl PartialOrd for MatchResult {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct MatchDatabase {
    entries: Vec<MatchEntry>,
}
impl MatchDatabase {
    pub fn from_cache(cache: &CardCache) -> Self {
        Self {
            entries: cache
                .par_iter()
                .flat_map_iter(|(_, card)| {
                    card.faces.iter().map(move |face| MatchEntry {
                        hash: ImageHash::from_bytes(&face.phash)
                            .expect("invalic image hash in card cache"),
                        card_id: card.id,
                        face: face.face,
                    })
                })
                .collect(),
        }
    }

    pub fn best_matches(&self, hash: &ImageHash) -> Vec<MatchResult> {
        let best = self
            .entries
            .par_iter()
            .fold(
                || ArrayVec::<MatchResult, NUM_MATCHES>::new(),
                |mut local, entry| {
                    insert_best(
                        &mut local,
                        MatchResult {
                            card_id: entry.card_id,
                            distance: hash.dist(&entry.hash),
                            face: entry.face,
                        },
                    );
                    local
                },
            )
            .reduce(
                || ArrayVec::<MatchResult, NUM_MATCHES>::new(),
                |mut a, b| {
                    for m in b {
                        insert_best(&mut a, m);
                    }
                    a
                },
            );

        best.into_iter().take(NUM_MATCHES).collect()
    }
}

fn insert_best(best: &mut ArrayVec<MatchResult, NUM_MATCHES>, candidate: MatchResult) {
    // Fast path while filling
    if best.len() < NUM_MATCHES {
        best.push(candidate);

        // Sort
        let mut i = best.len() - 1;
        while i > 0 && best[i] < best[i - 1] {
            best.swap(i, i - 1);
            i -= 1;
        }
        return;
    }

    // Candidate is not better than current worst
    if candidate >= best[NUM_MATCHES - 1] {
        return;
    }

    // Replace the worst
    best[NUM_MATCHES - 1] = candidate;

    // Sort
    let mut i = NUM_MATCHES - 1;
    while i > 0 && best[i] < best[i - 1] {
        best.swap(i, i - 1);
        i -= 1;
    }
}

pub fn get_hasher() -> Hasher {
    HasherConfig::new()
        .hash_alg(HashAlg::Gradient)
        .hash_size(16, 16)
        .to_hasher()
}
