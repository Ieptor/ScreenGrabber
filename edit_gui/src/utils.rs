use image::{load_from_memory, DynamicImage, GenericImageView};
use druid::{ImageBuf, Point};
use anyhow::{Result, Context, bail};
use native_dialog::{FileDialog};
use std::path::{Path, PathBuf};
use image::{Rgb, Rgba, RgbImage, RgbaImage, GenericImage};
use overlay_process::utils::save_into_clipboard;

pub fn resize_image(input_image: DynamicImage, target_size: (u32, u32)) -> DynamicImage {
    let (width, height) = input_image.dimensions();

    // Calculate scaling factors for width and height
    let width_scale = target_size.0 as f64 / width as f64;
    let height_scale = target_size.1 as f64 / height as f64;

    // Use the minimum of the two scaling factors to maintain aspect ratio
    let scale_factor = width_scale.min(height_scale);

    // Calculate the scaled dimensions
    let scaled_width = (width as f64 * scale_factor) as u32;
    let scaled_height = (height as f64 * scale_factor) as u32;

    // Resize the image
    let resized_image = input_image.resize_exact(scaled_width, scaled_height, image::imageops::FilterType::Lanczos3);

    resized_image
}

pub fn save_edited_image(image: DynamicImage, path: &str) -> Result<PathBuf> {
    let file_directory = FileDialog::new()
        .add_filter("PNG", &["png"])
        .add_filter("JPG", &["jpg"])
        .add_filter("GIF", &["gif"])
        .add_filter("JPEG", &["jpeg"])
        .set_location(Path::new(&path))
        .set_filename("default")
        .show_save_single_file()
        .context("Failed to show the save file dialog")?;

    if let Some(file_directory) = file_directory {
        let output_path = Path::new(&file_directory);
        
        //save dynamic image in output path
        image.save(output_path)
            .context("Failed to save the edited image")?;
        
        save_into_clipboard(output_path).context("Error copying into the clipboard")?;

        return Ok(output_path.to_path_buf());
    } else {
        bail!("Select a folder!")
    }
}

pub fn blend_images(base: DynamicImage, overlay: DynamicImage) -> DynamicImage {
    let (width, height) = base.dimensions();
    let mut result = DynamicImage::new_rgba8(width, height);

    for y in 0..height {
        for x in 0..width {
            let base_pixel = base.get_pixel(x, y);
            let overlay_pixel = overlay.get_pixel(x, y);

            let alpha = overlay_pixel.0[3] as f32 / 255.0;

            let blended_pixel = Rgba([
                ((1.0 - alpha) * base_pixel.0[0] as f32 + alpha * overlay_pixel.0[0] as f32) as u8,
                ((1.0 - alpha) * base_pixel.0[1] as f32 + alpha * overlay_pixel.0[1] as f32) as u8,
                ((1.0 - alpha) * base_pixel.0[2] as f32 + alpha * overlay_pixel.0[2] as f32) as u8,
                ((1.0 - alpha) * base_pixel.0[3] as f32 + alpha * overlay_pixel.0[3] as f32) as u8,
            ]);

            result.put_pixel(x, y, blended_pixel);
        }
    }

    result
}

pub fn apply_scaling(scaling_factors: (f64, f64), points: Vec<Point>) -> Vec<Point> {
    let (translation_x, translation_y) = scaling_factors;

    let converted_points: Vec<Point> = points
        .iter()
        .map(|&point| Point::new(point.x + translation_x, point.y + translation_y))
        .collect();

    converted_points
}

pub fn apply_scaling_to_point(scaling_factors: (f64, f64), point: Point) -> Point {
    let (translation_x, translation_y) = scaling_factors;
    Point::new(point.x + translation_x, point.y + translation_y)
}

pub fn calculate_distance(point1: &Point, point2: &Point) -> f64 {
    ((point2.x - point1.x).powi(2) + (point2.y - point1.y).powi(2)).sqrt()
}

pub fn calculate_radius(center: Point, point_on_circumference: Point) -> f64 {
    calculate_distance(&center, &point_on_circumference)
}