# Magic Vision

A Magic: The Gathering card identifier

## Outline

### Card cache builder

- Download bulk data json from scryfall
- Store in local file (jsonl)
- Download each image listed in the bulk data (rate limit)
  - Download border_crop versions
  - Local filename is card uuid
- Normalize:
  - Convert to grayscale
  - Resize to fixed dimensions?
  - Apply slight blur or normalization (mean subtraction)
- Hash each image using perceptual hash
- Add perceptual hash to database entries

### Webcam Pipeline

- Capture frame
- Card detection (OpenCV):
  - Convert to grayscale
  - Slight gaussian blur
  - Canny edge detection
  - Find contours
  - approxpolydp for 4 corners?
  - if no card found, skip frame
- Perspective normalization:
  - input 4 corners
  - imgproc::warpPerspective on pre-blur grayscale image (not canny) flatten
    to rectangle
- Output grayscale rectangle from warped perspective

### Matching and recognition

- On startup:
  - Load u64 phashes into memory (just a vec)
- Use same normalization steps from database building on input image
- generate perceptual hash
- compare linearly with rayon parallelization
  - maybe add some ANN/vector search later if comparison takes too long

### General

- Run webcam pipeline on its own thread and pass frames to the main thread in a
  small queue (dropping old frames when queue overflows).
- crossbeam::channel::bounded for message queue?
