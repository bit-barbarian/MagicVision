use image_hasher::{HashAlg, Hasher, HasherConfig};
use rayon::prelude::*;
use uuid::Uuid;

use crate::cache::card_cache::CardCache;

pub struct MatchEntry {
    pub hash: [u8; 32],
    pub card_id: Uuid,
    pub face: u8,
}

pub struct MatchDatabase {
    entries: Vec<MatchEntry>,
}
impl MatchDatabase {
    pub fn from_cache(cache: &CardCache) -> Self {
        MatchDatabase {
            entries: cache
                .par_iter()
                .flat_map_iter(|(_, card)| {
                    card.faces.iter().map(move |face| MatchEntry {
                        hash: face.phash,
                        card_id: card.id,
                        face: face.face,
                    })
                })
                .collect(),
        }
    }
}

pub fn get_hasher() -> Hasher {
    HasherConfig::new()
        .hash_alg(HashAlg::Gradient)
        .hash_size(16, 16)
        .to_hasher()
}
