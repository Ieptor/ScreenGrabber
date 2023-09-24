use std::borrow::Cow;
use druid::Rect;
use screenshots::Screen;
use std::{error, fs};

use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use std::path::{Path, PathBuf};

use image::{ImageBuffer, ImageFormat, ImageResult, Rgba, RgbImage};
use image::io::Reader as ImageReader;

extern crate clipboard;
extern crate image;

use clipboard::{ClipboardContext, ClipboardProvider};
use image::{DynamicImage, GenericImageView, ImageError};
use std::error::Error;
use std::io::Cursor;
use arboard::{Clipboard, ImageData};
use druid::platform_menus::mac::file::print;
use image::math::utils;
use native_dialog::{FileDialog, MessageDialog};

use screenshots::Image;
use thiserror::Error;

pub fn compute_window_size()-> (i32, i32, i32, i32) {
    let screens = Screen::all().unwrap();

    let mut leftmost = i32::MAX;
    let mut rightmost = i32::MIN;
    let mut topmost = i32::MAX;
    let mut bottommost = i32::MIN;
    for screen in &screens {
        leftmost = leftmost.min(screen.display_info.x);
        rightmost = rightmost.max(screen.display_info.x as i32 + (screen.display_info.width as f64 * screen.display_info.scale_factor as f64) as i32);
        topmost = topmost.min(screen.display_info.y);
        bottommost = bottommost.max(screen.display_info.y as i32 + (screen.display_info.height as f64 * screen.display_info.scale_factor as f64) as i32);
    }

    let width = rightmost - leftmost;
    let height = bottommost - topmost;

    return (width, height, leftmost, topmost)
}


pub fn read_config_file_savepath(file_path: &Path) -> io::Result<String> {
    let file = File::open(file_path).map_err(|err| {
        eprintln!("Error opening config file: {:?}", err);
        err
    })?;

    let reader = BufReader::new(file);

    // Read the first line of the file
    if let Some(Ok(path)) = reader.lines().next() {
        // If the line is not empty, return the path
        if !path.is_empty() {
            return Ok(path);
        }
    }

    // If the file is empty or there was an error reading, return an io::Error
    Err(io::Error::new(io::ErrorKind::InvalidData, "Config file is empty"))
}

pub fn capture_screenshot(selection: Rect, screen: Option<Screen>) {

    let selected_screen = screen.expect("No screen selected");

    //let mut screenshots = Vec::new();

    let scale_factor = selected_screen.display_info.scale_factor as f64;
    let unscaled_x0 = (selection.x0 - selected_screen.display_info.x as f64) / scale_factor;
    let unscaled_y0 = (selection.y0 - selected_screen.display_info.y as f64) / scale_factor;
    let _unscaled_x1 = (selection.x1 - selected_screen.display_info.x as f64) / scale_factor;
    let _unscaled_y1 = (selection.y1 - selected_screen.display_info.y as f64) / scale_factor;
    let unscaled_width = (selection.width() / scale_factor) as u32;
    let unscaled_height = (selection.height() / scale_factor) as u32;

    // Capture the screenshot using the adjusted coordinates on the unscaled screen
    let screen_shoot = selected_screen.capture_area(unscaled_x0 as i32, unscaled_y0 as i32, unscaled_width, unscaled_height);

    match screen_shoot{
        Ok(_) => {}
        Err(err) => {panic!("Error capturing screenshot: {:?}", err);} // handle error!
    };

    let path_str = Path::new("../config/config.txt");
    match read_config_file_savepath(path_str) {
        Ok(path) => {
            // select save directory starting from the default one
            let file_directory = FileDialog::new()
                .add_filter("PNG", &["png"])
                .add_filter("JPG", &["jpg"])
                .add_filter("GIF", &["gif"])
                .add_filter("JPEG", &["jpeg"])
                .set_location(Path::new(&path))
                .set_filename("default")
                .show_save_single_file()
                .unwrap();

            if let Some(file_directory) = file_directory {
                let output_path = Path::new(&file_directory);

                let png_bytes = screen_shoot.unwrap().to_png().expect("conversion error");
                let png_image = image::load_from_memory(&png_bytes).unwrap();

                let rgb_image: RgbImage = png_image.to_rgb8();
                let format = ImageFormat::from_path(output_path).expect("Conversion error");

                // save file into file system
                rgb_image.save_with_format(output_path, format).expect("conversion error");

                // save into the clipboard
                save_into_clipboard(output_path).expect("Error copying into the clipboard");

                println!("Image successfully saved!")
            } else {
                println!("Select a folder!");
            }

        },
        //Default
        Err(_) => {
            //fs::write("target/screenshot.png", buffer).unwrap();
            println!("NOT YET HANDLED");
        }
    }


}

pub fn capture_full_screen_screenshot (screen: Option<Screen>, all_screens: bool) {
    let selected_screen = screen.expect("No screen selected");
    let screen_shoot = selected_screen.capture();

}

pub fn save_into_clipboard(output_path: &Path) -> Result<(), arboard::Error> {
    let mut clipboard = Clipboard::new().unwrap();

    let image = image::open(output_path.clone()).expect("Error opening the image");

    // convert the image from DynamicImage to ImageData
    let img = ImageData {
        width: image.width() as usize,
        height: image.height() as usize,
        bytes: Cow::from(image.to_rgba8().into_raw())
    };
    // write into the clipboard
    match clipboard.set_image(img) {
        Ok(_) => {}
        Err(_) => { return Err(arboard::Error::ClipboardNotSupported) }
    }
    Ok(())
}
