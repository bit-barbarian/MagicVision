# MagicVision

The easiest way to import your physical decks into
[Moxfield](https://moxfield.com/),
[Archidekt](https://archidekt.com/),
[MtgDesktopCompanion](https://github.com/nicho92/MtgDesktopCompanion),
or any other collection tracker!

<https://github.com/user-attachments/assets/54fded23-b5d5-4c2f-bd4b-61ece0fc191b>

## Features

- Real-time webcam card recognition and matching
- Entirely local recognition after initial setup
- Native desktop GUI built with [egui](https://github.com/emilk/egui)
- Review and edit the decklist as you go
- Export decklists in multiple formats
  (see [Supported formats](#supported-export-formats))

> [!NOTE]
> If you use a collection tracker with a format not included in MagicVision,
> please leave a feature request with the format you use!

## Installation

### Windows

1. Download the latest [Release](https://github.com/bit-barbarian/MagicVision/releases/latest).
   Ensure the OpenCV dlls remain in the same folder as `magicvision` or
   are discoverable on your path.

### Mac

1. `brew install opencv`
2. Download the latest [Release](https://github.com/bit-barbarian/MagicVision/releases/latest)

### Linux

1. Install OpenCV v5.0.0
2. Download the latest [Release](https://github.com/bit-barbarian/MagicVision/releases/latest)

### From source

1. Install OpenCV v5.0.0 (4.13.0 for Windows),
   clang (part of llvm), and
   [rust](https://rust-lang.org/tools/install/).
2. `git clone`
3. `cd MagicVision`
4. Windows only: `git switch master-windows`
5. `cargo run --bin magicvision` (or `--bin refresh-database`)

## Quick Start

1. Run the `refresh-database` executable once to build the card cache
   (~15-30m first run, ~30s on subsequent runs).
2. Run `magicvision`.
3. Scan your cards.
4. Click the `+` button and click the foil type of your card to add it to the
   current decklist.
5. Once you've added all your cards, ensure you have the correct format
   selected in the top left.
6. Click "save", choose a file name and location.
7. Done!

## Usage

If it is your first time using MagicVision, run the `refresh-database`
executable. This will:

1. Pull the latest cards and changes from Scryfall and download images of
   every card face missing from your local cache.
2. Generate a [perceptual hash](https://en.wikipedia.org/wiki/Perceptual_hashing)
   of each Magic card and store it alongside the Scryfall card data.

> [!NOTE]
> The cache uses ~500MB for Scryfall data and ~12GB for images.
> Images and data are stored locally so that recognition is both fast and
> available offline.

`refresh-database` will take a while to run depending on your download speed and
CPU power. The first run takes much longer than subsequent runs (20-30m vs 30-60s).
It waits 20ms between downloads to be respectful to Scryfall's server.
It will not re-download cards. If it is interrupted, it will only download
missing cards next time it is run.

> [!TIP]
> If you want to re-generate MagicVision's card cache data, you can delete its cache
> file and run `refresh-database`. If you are not missing any card images, this
> will only download the newest Scryfall bulk data, if available
> ([updated every 12-24 hours](https://scryfall.com/docs/api/bulk-data)), and
> remake your local cache.

Now you're ready to run `magicvision`! Make sure you have a webcam available
and start scanning!

Click the `+` button next to the Best Match card or any card in
the "Other matches" section to add it to your current decklist on the right.

Drag the number slider next to each card in the decklist to change the
number of copies of that card in the deck.

Click the `-` button next to any card in the list to remove all copies
of that card from the deck.

Sort the list at any time by clicking `Sort` at the top of the decklist panel.

Once you've scanned in your whole deck, make sure you have the correct
format selected in the top left.

Click `Save` at the top of the decklist panel and choose a file name and location.

Your deck is now ready to be imported to your favorite collection manager!

## Supported export formats

- Moxfield
- Archidekt

## Troubleshooting

### Stuck on "Loading..."

Ensure you have a working webcam available.

### OpenCV or .dll errors

#### OpenCV Windows

If you've moved the program files around, make sure that the opencv dlls
are always in the same folder as the `magicvision` executable.

#### OpenCV Unix

Ensure that you have version 5.0 of OpenCV installed and available

### Recognition not working?

1. Make sure you have a solid background that stands out against the cards.
2. Strong shadows (and similar things) can break the outline of the card
   that the detection algorithm uses to find a card in the camera frame.
3. Sleeved cards will be detected but will likely harm card matching
   capabilities, especially if the inside of the sleeve is not black.

## Roadmap

- Now: Improve image processing performance
- Next: Support configurable perceptual hashing algorithms and hash sizes
- Future: Allow custom cache/data locations
