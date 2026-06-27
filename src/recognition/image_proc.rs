use opencv::{
    Result,
    core::{AlgorithmHint::ALGO_HINT_DEFAULT, BORDER_DEFAULT, Mat, Point, Point2f, Size, Vector},
    imgproc,
};

use crate::recognition::card_detection::CardDetection;

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

pub fn detect_card(frame: &Mat) -> Result<Option<CardDetection>> {
    let mut contours = Vector::<Vector<Point>>::new();
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
    let mut largest_contour: Option<CardDetection> = None;

    for contour in contours {
        let area = imgproc::contour_area(&contour, false)?;

        // Filter out small contour lines
        if area < 3000.0 {
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

        // Filter out non-rectantular approximated polygons
        if approx.len() != 4 {
            continue;
        };
        assert_eq!(approx.len(), 4);

        // Convert approximated rectangle to f32 point array
        let mut approx_arr: [Point2f; 4] = [Point2f::default(); 4];
        for (i, p) in approx.iter().enumerate() {
            approx_arr[i] = p;
        }

        // Select the largest remaining contour (closest to camera)
        match &largest_contour {
            Some(largest) => {
                if area > largest.area {
                    largest_contour = Some(CardDetection {
                        contour,
                        corners: approx_arr,
                        area,
                    })
                }
            }
            None => {
                largest_contour = Some(CardDetection {
                    contour,
                    corners: approx_arr,
                    area,
                })
            }
        }
    }

    match &largest_contour {
        Some(largest) => Ok(Some(CardDetection::new_with_ordered_corners(
            &largest.contour,
            largest.corners,
            largest.area,
        ))),
        None => Ok(None),
    }
}
