# MagicVision

The easiest way to import your physical decks into
[Moxfield](https://moxfield.com/),
[Archidekt](https://archidekt.com/),
[MtgDesktopCompanion](https://github.com/nicho92/MtgDesktopCompanion),
or any other collection tracker!

## Features

- Real-time webcam scanning
- Automatic Magic: The Gathering card recognition
- Native desktop GUI built with egui
- Export decklists in multiple formats\*
- Review and edit the decklist before exporting

\*If you use a collection tracker that uses a format not
included in MagicVision, leave a feature request with the format you need!

## Installation

### Windows

1. Download windows asset from the latest [Release](https://github.com/bit-barbarian/MagicVision/releases/latest).
   Ensure the opencv dlls remain in the same folder as `magicvision` or
   are discoverable on your path.

### Mac

1. Install opencv: `brew install opencv`
2. Download macOS asset from the latest [Release](https://github.com/bit-barbarian/MagicVision/releases/latest)

### Linux

1. Install opencv >= v4.11 (tested with 4.13) either through your distribution's
   package manager or from source and ensure the libraries are visible on your path.
2. Download linux asset from the latest [Release](https://github.com/bit-barbarian/MagicVision/releases/latest)

### From source

1. Install opencv >= v4.11 (tested with 4.13), clang (part of llvm), and
   [rust](https://rust-lang.org/tools/install/).
2. `git clone`
3. `cd MagicVision`
4. `cargo run --bin magicvision` (or `--bin refresh-database`)

## Usage

If it is your first time using MagicVision, run `refresh-database`. This will:

1. Pull the latest changes from Scryfall and download images of each card face.
2. Generate a [perceptual hash](https://en.wikipedia.org/wiki/Perceptual_hashing)
   of each magic card and store it alongside the Scryfall card data.

> [!NOTE]
> The total space used for cache data is ~12GB.

`refresh-database` will take a while to run depending on your download speed and
CPU power. It waits 20ms between downloads to be respectful to Scryfall's server.
It will not re-download cards. If it is interrupted, it will only download
missing cards next time it is run.

> [!TIP]
> If you want to re-generate MagicVision's card cache data, you can delete its cache
> file and run `refresh-database`. If you are not missing any card images, this
> will only download the newest Scryfall bulk data, if available
> ([updated every 12-24 hours](https://scryfall.com/docs/api/bulk-data)), and
> remake your local cache.

Now you're ready to run `magicvision`! Make sure you have a webcam available
and get scanning!

## Roadmap

- Add a demo video to README
- Add option flags to use different types of perceptual hashes, or change data directories
- Integrate [rgb](https://crates.io/crates/rgb)
  to increase format conversion efficiency
