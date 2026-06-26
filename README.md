# Magic Vision

A Magic: The Gathering card identifier

## Outline

### Card cache builder

- Download bulk data json from scryfall
- Store in local file (ndjson)
- Download each image listed in the bulk data (technically no rate limit on scryfall.io)
  - Download border_crop versions, fall back to normal if border_crop doesn't exist
  - Local filename is {card_id}\_{face_number}
  - 0 = front, 1 = back, >1 = other
- Normalizing is done by image_hasher crate
- Hash each image using perceptual hash
- Add perceptual hash to cache entries
- Persist cache on fs

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
  - Load phashes into memory
- Use OpenCV to identify a card in input image
- generate perceptual hash with same hashing function used to build cache.
- compare input card hash to hash cache with rayon parallelization
  - maybe add some ANN/vector search later if comparison takes too long

### General

- Run webcam pipeline on its own thread and pass frames to the main thread in a
  small queue (dropping old frames when queue overflows).
- crossbeam::channel::bounded for message queue?

```ascii
              Camera Thread
          ┌────────────────────┐
          │ VideoCapture.read  │
          └─────────┬──────────┘
                    │
            bounded channel (2)
                    │
                    ▼
           Recognition Thread
    ┌──────────────────────────────┐
    │ grayscale                    │
    │ blur                         │
    │ canny                        │
    │ contours                     │
    │ perspective warp             │
    │ perceptual hash              │
    │ parallel MatchEntry search   │
    └──────────────┬───────────────┘
                   │
                   ▼
          Draw result on frame
                   │
                   ▼
                imshow()
```
