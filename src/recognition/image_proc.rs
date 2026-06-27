use std::cmp::Ordering;

use opencv::{
    Result,
    core::{
        AlgorithmHint::ALGO_HINT_DEFAULT, BORDER_DEFAULT, Mat, Point, Point2f, Size, Vector,
        VectorToVec,
    },
    imgproc,
};

struct RawContour {
    points: Vector<Point2f>,
    area: f64,
}

pub struct CardDetection {
    pub contour: Vector<Point2f>,
    pub corners: [Point2f; 4], // Ordered with index 0 top-left
    pub area: f64,
}
impl CardDetection {
    pub fn new(contour: &Vector<Point2f>, area: f64) -> Self {
        debug_assert_eq!(contour.len(), 4);
        CardDetection {
            contour: contour.clone(),
            corners: CardDetection::order_corners(contour),
            area,
        }
    }

    fn order_corners(contour: &Vector<Point2f>) -> [Point2f; 4] {
        // Work on native vec to use Rust sorting functions
        let mut points = contour.to_vec();
        assert_eq!(points.len(), 4);

        let centroid = Point2f::new(
            points.iter().map(|p| p.x).sum::<f32>() / 4.0,
            points.iter().map(|p| p.y).sum::<f32>() / 4.0,
        );

        // Sort by angle around centroid.
        points.sort_by(|a, b| {
            let angle_a = (a.y - centroid.y).atan2(a.x - centroid.x);
            let angle_b = (b.y - centroid.y).atan2(b.x - centroid.x);

            angle_a.partial_cmp(&angle_b).unwrap_or(Ordering::Equal)
        });

        // Rotate so top-left comes first
        let tl = points
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                (a.x + a.y)
                    .partial_cmp(&(b.x + b.y))
                    .unwrap_or(Ordering::Equal)
            })
            .unwrap()
            .0;
        points.rotate_left(tl);

        // Ensure clockwise ordering
        let signed_area: f32 = points
            .iter()
            .zip(points.iter().cycle().skip(1))
            .take(4)
            .map(|(a, b)| a.x * b.y - b.x * a.y)
            .sum();
        if signed_area > 0.0 {
            points.swap(1, 3);
        }

        [points[0], points[1], points[2], points[3]]
    }

    fn order_corners_simple(contour: Vector<Point2f>) -> [Point2f; 4] {
        let mut points = contour.to_vec();
        assert_eq!(points.len(), 4);

        let mut tl = points[0];
        let mut tr = points[1];
        let mut bl = points[2];
        let mut br = points[3];

        let mut min_sum = f32::MAX;
        let mut max_sum = f32::MIN;
        let mut min_diff = f32::MAX;
        let mut max_diff = f32::MIN;

        for p in points {
            let sum = p.x + p.y;
            let diff = p.x - p.y;

            if sum < min_sum {
                min_sum = sum;
                tl = p;
            }
            if sum > max_sum {
                max_sum = sum;
                br = p;
            }
            if diff < min_diff {
                min_diff = diff;
                tr = p;
            }
            if diff > max_diff {
                max_diff = diff;
                bl = p;
            }
        }

        [tl, tr, br, bl]
    }
}

pub fn preprocess(frame: &Mat) -> Result<Mat> {
    let mut gray = Mat::default();

    imgproc::cvt_color(
        frame,
        &mut gray,
        imgproc::COLOR_BGR2GRAY,
        0,
        ALGO_HINT_DEFAULT,
    )?;

    let mut blur = Mat::default();

    imgproc::gaussian_blur(
        &gray,
        &mut blur,
        Size::new(3, 3),
        0.0,
        0.0,
        BORDER_DEFAULT,
        ALGO_HINT_DEFAULT,
    )?;

    let mut edges = Mat::default();

    imgproc::canny(&blur, &mut edges, 80.0, 130.0, 3, true)?;

    Ok(edges)
}

pub fn get_contour(frame: &Mat) -> Result<Option<CardDetection>> {
    let mut contours = Vector::<Vector<Point2f>>::new();
    let mut hierarchy = Mat::default();

    imgproc::find_contours_with_hierarchy(
        frame,
        &mut contours,
        &mut hierarchy,
        imgproc::RETR_EXTERNAL,
        imgproc::CHAIN_APPROX_SIMPLE,
        Point::new(0, 0),
    )?;

    // Eventually return largest contour (closest card to camera)
    let mut largest_contour: Option<RawContour> = None;

    for contour in contours {
        let area = imgproc::contour_area(&contour, false)?;

        // Filter out small contour lines
        if area < 1000.0 {
            continue;
        };

        // Filter out contours that are not the same ratio as a magic card
        let perimeter = imgproc::arc_length(&contour, true)?;
        let ratio = perimeter / area.sqrt();
        if ratio > 4.35 {
            continue;
        };

        let mut approx = Vector::<Point2f>::new();
        imgproc::approx_poly_dp(&contour, &mut approx, 0.013 * perimeter, true)?;

        // Filter out non-rectantular contours
        if approx.len() != 4 {
            continue;
        };

        // Select the largest remaining contour (closest to camera)
        match &largest_contour {
            Some(largest) => {
                if area > largest.area {
                    largest_contour = Some(RawContour {
                        points: approx,
                        area,
                    })
                }
            }
            None => {
                largest_contour = Some(RawContour {
                    points: approx,
                    area,
                })
            }
        }
    }

    match &largest_contour {
        Some(largest) => Ok(Some(CardDetection::new(&largest.points, largest.area))),
        None => Ok(None),
    }
}
