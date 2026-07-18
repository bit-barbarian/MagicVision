use egui::ColorImage;
use image::{DynamicImage, ImageBuffer, Rgb};
use image_hasher::{Hasher, ImageHash};
use opencv::{
    Result,
    core::{
        AlgorithmHint::ALGO_HINT_DEFAULT, BORDER_CONSTANT, BORDER_DEFAULT, Mat, MatTraitConst,
        MatTraitConstManual, Point, Point2f, Size, Vector,
    },
    geometry, imgproc,
};

use crate::{recognition::card_detection::CardDetection, types::DynResult};

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

    let mut dilated = Mat::default();
    let kernel = imgproc::get_structuring_element(
        imgproc::MORPH_ELLIPSE,
        Size::new(3, 3),
        Point::new(-1, -1),
    )?;
    imgproc::dilate(
        &edges,
        &mut dilated,
        &kernel,
        Point::new(-1, -1),
        2,
        BORDER_CONSTANT,
        imgproc::morphology_default_border_value()?,
    )?;

    Ok(dilated)
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
        let area = geometry::contour_area(&contour, false)?;

        // Filter out small contour lines
        if area < 3000.0 {
            continue;
        };

        // Filter out contours that are not the same ratio as a magic card
        let perimeter = geometry::arc_length(&contour, true)?;
        let ratio = perimeter / area.sqrt();
        if ratio > 4.35 {
            continue;
        };

        let mut approx = Vector::<Point2f>::new();
        geometry::approx_poly_dp(&contour, &mut approx, 0.013 * perimeter, true)?;

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

pub fn hash_mat(mat: &Mat, hasher: &Hasher) -> DynResult<ImageHash> {
    let image = mat_to_dynamic_image(mat)?;
    Ok(hasher.hash_image(&image))
}

pub fn hash_image(image: &DynamicImage, hasher: &Hasher) -> DynResult<ImageHash> {
    Ok(hasher.hash_image(image))
}

fn mat_to_dynamic_image(mat: &Mat) -> opencv::Result<DynamicImage> {
    // Convert from BGR (opencv) to RGB (image)
    let mut rgb = Mat::default();
    imgproc::cvt_color(mat, &mut rgb, imgproc::COLOR_BGR2RGB, 0, ALGO_HINT_DEFAULT)?;

    let width = rgb.cols();
    let height = rgb.rows();

    let bytes = rgb.data_bytes()?.to_vec();

    let image = ImageBuffer::<Rgb<u8>, Vec<u8>>::from_raw(width as u32, height as u32, bytes)
        .expect("unexpected image dimensions");

    Ok(DynamicImage::ImageRgb8(image))
}

pub fn mat_to_color_image(mat: &Mat) -> opencv::Result<ColorImage> {
    // Convert from BGR (opencv) to RGB (image)
    let mut rgba = Mat::default();
    imgproc::cvt_color(
        mat,
        &mut rgba,
        imgproc::COLOR_BGR2RGBA,
        0,
        ALGO_HINT_DEFAULT,
    )?;

    let width = rgba.cols();
    let height = rgba.rows();
    let bytes = rgba.data_bytes()?.to_vec();

    let image = ColorImage::from_rgba_unmultiplied([width as usize, height as usize], &bytes);
    Ok(image)
}
