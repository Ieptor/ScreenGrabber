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


pub struct Stroke {
    points: Vec<Point>,
    color: Rgba<u8>,
    width: u32,
}

impl Stroke {
    pub fn new(points: Vec<Point>, color: Rgba<u8>, width: u32) -> Self {
        Stroke { points, color, width }
    }

    pub fn draw(&self, size: (u32, u32)) -> RgbaImage {
        let mut image = RgbaImage::new(size.0, size.1); 
    
        for point in &self.points {
            for i in 0..self.width {
                for j in 0..self.width {
                    let x = point.x + (i as f64);
                    let y = point.y + (j as f64);
    
                    if x < image.width().into() && y < image.height() as f64 {
                        image.put_pixel(x as u32, y as u32, self.color);
                    }
                }
            }
        }
    
        image
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