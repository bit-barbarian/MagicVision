use opencv::{
    Result,
    core::{Mat, Point, Point2f, Scalar, Size, Vector},
    imgproc,
};

pub struct CardDetection {
    pub contour: Vector<Point>,
    pub corners: [Point2f; 4], // Ordered clockwise with index 0 top-left
    pub area: f64,
}
impl CardDetection {
    pub fn new_with_ordered_corners(
        contour: &Vector<Point>,
        corners: [Point2f; 4],
        area: f64,
    ) -> Self {
        debug_assert_eq!(corners.len(), 4);
        CardDetection {
            contour: contour.clone(),
            corners: CardDetection::order_corners(corners),
            area,
        }
    }

    fn order_corners(corners: [Point2f; 4]) -> [Point2f; 4] {
        let points = corners;

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
            let diff = p.y - p.x;

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

    pub fn draw(&self, frame: &mut Mat) -> Result<()> {
        let mut contours = Vector::<Vector<Point>>::new();
        contours.push(self.contour.clone());
        // Draw contour
        imgproc::draw_contours(
            frame,
            &contours,
            -1,
            Scalar::new(0.0, 255.0, 0.0, 0.0),
            2,
            imgproc::LINE_AA,
            &opencv::core::no_array(),
            i32::MAX,
            Point::new(0, 0),
        )?;

        // Draw polygon
        for i in 0..4 {
            let a = self.corners[i];
            let b = self.corners[(i + 1) % 4];

            imgproc::line(
                frame,
                Point::new(a.x as i32, a.y as i32),
                Point::new(b.x as i32, b.y as i32),
                Scalar::new(0.0, 0.0, 255.0, 0.0),
                1,
                imgproc::LINE_AA,
                0,
            )?;
        }

        // Draw corner markers
        for (i, corner) in self.corners.iter().enumerate() {
            imgproc::circle(
                frame,
                Point::new(corner.x as i32, corner.y as i32),
                6,
                Scalar::new(0.0, 0.0, 255.0, 0.0),
                -1,
                imgproc::LINE_AA,
                0,
            )?;

            imgproc::put_text(
                frame,
                &i.to_string(),
                Point::new(corner.x as i32 + 10, corner.y as i32 - 10),
                imgproc::FONT_HERSHEY_SIMPLEX,
                2.0,
                Scalar::new(255.0, 0.0, 0.0, 0.0),
                2,
                imgproc::LINE_AA,
                false,
            )?;
        }

        // Draw area number
        imgproc::put_text(
            frame,
            &format!("Area: {:0}", self.area),
            Point::new(self.corners[0].x as i32, self.corners[0].y as i32 - 30),
            imgproc::FONT_HERSHEY_SIMPLEX,
            0.9,
            Scalar::new(0.0, 255.0, 0.0, 0.0),
            2,
            imgproc::LINE_AA,
            false,
        )?;

        Ok(())
    }

    pub fn warp(&self, image: &Mat) -> Result<Mat> {
        const WIDTH: i32 = 480;
        const HEIGHT: i32 = 680;

        let mut dst = Vector::<Point2f>::new();
        dst.push(Point2f::new(0.0, 0.0));
        dst.push(Point2f::new(WIDTH as f32 - 1.0, 0.0));
        dst.push(Point2f::new(WIDTH as f32 - 1.0, HEIGHT as f32 - 1.0));
        dst.push(Point2f::new(0.0, HEIGHT as f32 - 1.0));

        // Check if card is sideways and transform appropriately
        let mut src = Vector::<Point2f>::new();
        let top = euclidean_distance(self.corners[0], self.corners[1]);
        let left = euclidean_distance(self.corners[0], self.corners[3]);

        if top < left {
            // card is portrait
            for p in self.corners {
                src.push(p);
            }
        } else {
            // card is landscape
            let mut rotated_corners = self.corners;
            rotated_corners.rotate_right(1);
            for p in rotated_corners {
                src.push(p);
            }
        }

        let transform = imgproc::get_perspective_transform(&src, &dst, opencv::core::DECOMP_LU)?;

        let mut warped = Mat::default();
        imgproc::warp_perspective(
            image,
            &mut warped,
            &transform,
            Size::new(WIDTH, HEIGHT),
            imgproc::INTER_LINEAR,
            opencv::core::BORDER_CONSTANT,
            Scalar::default(),
        )?;
        Ok(warped)
    }
}

fn euclidean_distance(a: Point2f, b: Point2f) -> f32 {
    (a.x - b.x).hypot(a.y - b.y)
}
