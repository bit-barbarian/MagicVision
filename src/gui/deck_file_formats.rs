use crate::cache::card_cache::CachedCard;
use std::cmp::Ordering;

#[derive(PartialEq)]
pub enum DeckFileFormat {
    Moxfield,
    Archidekt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FoilType {
    None,
    Foil,
    Etched,
}

pub struct DeckCard {
    pub card: CachedCard,
    pub count: u32,
    pub foil_type: FoilType,
}

impl PartialEq for DeckCard {
    fn eq(&self, other: &Self) -> bool {
        self.card.id == other.card.id && self.foil_type == other.foil_type
    }
}

impl Eq for DeckCard {}

impl PartialOrd for DeckCard {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DeckCard {
    fn cmp(&self, other: &Self) -> Ordering {
        self.card
            .name
            .cmp(&other.card.name)
            .then_with(|| self.card.id.cmp(&other.card.id))
            .then_with(|| self.foil_type.cmp(&other.foil_type))
    }
}

pub struct DeckList {
    pub cards: Vec<DeckCard>,
}

impl DeckList {
    pub fn add(&mut self, card: CachedCard, foil_type: FoilType) {
        // If card already exists in decklist, add 1 to count
        for dc in &mut self.cards {
            if dc.card.id == card.id && dc.foil_type == foil_type {
                dc.count += 1;
                return;
            }
        }

        // Otherwise, add new entry
        self.cards.push(DeckCard {
            card,
            count: 1,
            foil_type,
        })
    }

    pub fn sort(&mut self) {
        self.cards.sort();
    }
}

pub fn format_file_headers(file_format: &DeckFileFormat) -> String {
    match file_format {
        DeckFileFormat::Moxfield => {
            String::from("\"Count\",\"Name\",\"Edition\",\"Collector Number\",\"Foil\"\n")
        }
        DeckFileFormat::Archidekt => String::from(""),
    }
}

pub fn format_line(file_format: &DeckFileFormat, dc: &DeckCard) -> String {
    match file_format {
        DeckFileFormat::Moxfield => {
            format!(
                "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"\n",
                dc.count,
                dc.card.name,
                dc.card.set,
                dc.card.number,
                match dc.foil_type {
                    FoilType::None => "",
                    FoilType::Foil => "foil",
                    FoilType::Etched => "etched",
                }
            )
        }
        DeckFileFormat::Archidekt => format!("{} {}\n", dc.count, dc.card.name),
    }
}
