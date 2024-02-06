use druid::{Point};
use image::{Rgba, RgbaImage};

pub struct Stroke {
    points: Vec<Point>,
    color: Rgba<u8>,
    width: u32,
}

impl Stroke {
    pub fn new(points: Vec<Point>, color: Rgba<u8>, width: u32) -> Self {
        Stroke { points, color, width }
    }

    pub fn new_empty() -> Self {
        Stroke {
            points: Vec::new(),
            color: Rgba([0, 0, 0, 255]),
            width: 2,
        }
    }

    pub fn draw(&self, size: (u32, u32)) -> RgbaImage {
        let mut image = RgbaImage::new(size.0, size.1); 
    
        // Iterate over each pair of adjacent points
        for i in 0..self.points.len() - 1 {
            let p1 = self.points[i];
            let p2 = self.points[i + 1];
            
            // Calculate the direction vector between the points
            let delta_x = p2.x - p1.x;
            let delta_y = p2.y - p1.y;
            
            // Calculate the length of the line segment
            let length = (delta_x.powi(2) + delta_y.powi(2)).sqrt();
            
            // Normalize the direction vector
            let step_x = delta_x / length;
            let step_y = delta_y / length;
            
            // Iterate over the length of the line segment
            for t in 0..=length as u32 {
                // Calculate the position along the line segment
                let x = p1.x + step_x * t as f64;
                let y = p1.y + step_y * t as f64;
                
                // Draw a pixel at the calculated position
                for i in 0..self.width {
                    for j in 0..self.width {
                        let px = (x + i as f64) as u32;
                        let py = (y + j as f64) as u32;
                        
                        if px < image.width() && py < image.height() {
                            image.put_pixel(px, py, self.color);
                        }
                    }
                }
            }
        }
        image
    }

    pub fn draw_circle(&self, center: Point, radius: f64, size: (u32, u32)) -> RgbaImage {
        let mut image = RgbaImage::new(size.0, size.1);

        let mut x = radius as i64;
        let mut y = 0i64;
        let mut err = 0i64;

        while x >= y {
            for i in 0..self.width {
                // Octant 1
                if ((center.x + x as f64 + i as f64) as u32) < size.0 && ((center.y + y as f64) as u32) < size.1 {
                    image.put_pixel((center.x + x as f64 + i as f64) as u32, (center.y + y as f64) as u32, self.color);
                }
                if ((center.x + y as f64) as u32) < size.0 && ((center.y + x as f64 + i as f64) as u32) < size.1 {
                    image.put_pixel((center.x + y as f64) as u32, (center.y + x as f64 + i as f64) as u32, self.color);
                }

                // Octant 2
                if ((center.x - y as f64) as u32) < size.0 && ((center.y + x as f64 + i as f64) as u32) < size.1 {
                    image.put_pixel((center.x - y as f64) as u32, (center.y + x as f64 + i as f64) as u32, self.color);
                }
                if ((center.x - x as f64 - i as f64) as u32) < size.0 && ((center.y + y as f64) as u32) < size.1 {
                    image.put_pixel((center.x - x as f64 - i as f64) as u32, (center.y + y as f64) as u32, self.color);
                }

                // Octant 3
                if ((center.x - x as f64 - i as f64) as u32) < size.0 && ((center.y - y as f64) as u32) < size.1 {
                    image.put_pixel((center.x - x as f64 - i as f64) as u32, (center.y - y as f64) as u32, self.color);
                }
                if ((center.x - y as f64) as u32) < size.0 && ((center.y - x as f64 - i as f64) as u32) < size.1 {
                    image.put_pixel((center.x - y as f64) as u32, (center.y - x as f64 - i as f64) as u32, self.color);
                }

                // Octant 4
                if ((center.x + y as f64) as u32) < size.0 && ((center.y - x as f64 - i as f64) as u32) < size.1 {
                    image.put_pixel((center.x + y as f64) as u32, (center.y - x as f64 - i as f64) as u32, self.color);
                }
                if ((center.x + x as f64 + i as f64) as u32) < size.0 && ((center.y - y as f64) as u32) < size.1 {
                    image.put_pixel((center.x + x as f64 + i as f64) as u32, (center.y - y as f64) as u32, self.color);
                }
            }

            y += 1;
            if err <= 0 {
                err += 2 * y + 1;
            }
            if err > 0 {
                x -= 1;
                err -= 2 * x + 1;
            }
        }

        image
    }

    pub fn draw_enlarging_rectangle(&self, initial_position: Point, current_position: Point, size: (u32, u32)) -> RgbaImage {
        let mut image = RgbaImage::new(size.0, size.1);
    
        // Calculate the top-left and bottom-right corners of the rectangle
        let top_left = Point::new(initial_position.x.min(current_position.x), initial_position.y.min(current_position.y));
        let bottom_right = Point::new(initial_position.x.max(current_position.x), initial_position.y.max(current_position.y));
    
        let thickness = self.width + 1;
        // Draw the rectangle using the calculated corners
        for x in (top_left.x as u32)..=(bottom_right.x as u32) {
            for y in (top_left.y as u32)..=(bottom_right.y as u32) {
                if x < size.0 && y < size.1 {
                    // Draw the border of the rectangle
                    let left_border = top_left.x as u32 + thickness - 1;
                    let right_border = bottom_right.x as u32 - thickness+ 1;
                    let top_border = top_left.y as u32 + thickness - 1;
                    let bottom_border = bottom_right.y as u32 - thickness + 1;
    
                    if x >= top_left.x as u32 && x <= bottom_right.x as u32 && y >= top_left.y as u32 && y <= bottom_right.y as u32 {
                        // Fill the inside of the rectangle
                        if x < left_border || x > right_border || y < top_border || y > bottom_border {
                            // Draw only the border of the rectangle
                            image.put_pixel(x, y, self.color);
                        }
                    }
                }
            }
        }
    
        image
    }

    pub fn draw_enlarging_triangle(&self, initial_position: Point, current_position: Point, size: (u32, u32)) -> RgbaImage {
        let mut image = RgbaImage::new(size.0, size.1);
    
        // Calculate the length of the sides of the equilateral triangle
        let side_length = ((current_position.x - initial_position.x).powi(2) + (current_position.y - initial_position.y).powi(2)).sqrt();
    
        // Calculate the vertices of the equilateral triangle
        let (x1, y1) = (initial_position.x, initial_position.y - side_length / (3.0 as f64).sqrt());
        let (x2, y2) = (initial_position.x - side_length / 2.0, initial_position.y + side_length / (2.0 as f64).sqrt());
        let (x3, y3) = (initial_position.x + side_length / 2.0, initial_position.y + side_length / (2.0 as f64).sqrt());
    
        // Draw the triangle using the calculated vertices
        self.draw_line(&mut image, Point::new(x1, y1), Point::new(x2, y2), size);
        self.draw_line(&mut image, Point::new(x2, y2), Point::new(x3, y3), size);
        self.draw_line(&mut image, Point::new(x3, y3), Point::new(x1, y1), size);
    
        image
    }
    
    fn draw_line(&self, image: &mut RgbaImage, start: Point, end: Point, size: (u32, u32)) {
        // Bresenham's line algorithm for drawing lines
        let mut x0 = start.x.round() as i32;
        let mut y0 = start.y.round() as i32;
        let x1 = end.x.round() as i32;
        let y1 = end.y.round() as i32;
    
        let dx = (x1 - x0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 - y0).abs();
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
    
        // Thickness of the line
        let thickness = self.width - 1;
    
        loop {
            // Check if the pixel coordinates are within the image bounds
            if x0 >= 0 && x0 < size.0 as i32 && y0 >= 0 && y0 < size.1 as i32 {
                // Draw thicker border
                for i in 0..= thickness {
                    let offset_i = i as i32 - thickness as i32;
                    for j in 0..= thickness{
                        let offset_j = j as i32 - thickness as i32;
                        let new_x = x0 + offset_i;
                        let new_y = y0 + offset_j;
                        // Check if the new coordinates are still within the image bounds
                        if new_x >= 0 && new_x < size.0 as i32 && new_y >= 0 && new_y < size.1 as i32 {
                            image.put_pixel(new_x as u32, new_y as u32, self.color);
                        }
                    }
                }
            }
            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }
}