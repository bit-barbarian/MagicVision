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

### Recognition and Matching

- On thread startup:
  - Load phashes into memory
- Card detection (OpenCV):
  - Convert to grayscale
  - Slight gaussian blur
  - Canny edge detection
  - dilate edges
  - Find contours
  - approxpolydp filter for 4 corners and card ratio
  - if no card found, skip frame
- Perspective normalization:
  - Input 4 corners
  - Output rectangle from original unprocessed frame with warped perspective points
    (image_hasher handles preprocessing, same as cache builder)
- generate perceptual hash with same hashing function used to build cache.
- compare input card hash to hash cache with rayon parallelization
  - maybe add some ANN/vector search later if comparison takes too long
  - Hierarchical navigable small world
- Return:
  - Original frame drawn over with contour highlighting detected card
  - MatchEntry

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
            UI (main) thread
               imshow()
```
