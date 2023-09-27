use std::borrow::Cow;
use druid::Rect;
use screenshots::Screen;
use std::{error, fs};
use std::any::Any;

use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use std::path::{Path, PathBuf};

use image::{GenericImage, ImageBuffer, ImageFormat, ImageResult, Rgba, RgbImage};
use image::io::Reader as ImageReader;

extern crate clipboard;
extern crate image;

use clipboard::{ClipboardContext, ClipboardProvider};
use image::{DynamicImage, GenericImageView, ImageError};
use std::error::Error;
use std::io::Cursor;
use std::sync::mpsc::RecvError;
use arboard::{Clipboard, ImageData};
use druid::platform_menus::mac::file::print;
use image::math::utils;
use native_dialog::{FileDialog, MessageDialog, MessageType};

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

fn read_config_file_savepath(file_path: &Path) -> io::Result<String> {
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

pub fn capture_screenshot(mut selection: Rect, screen: Option<Screen>, translation_factor: i32) {

    // build a Vec<Screen> without the screens we are sure are not needed
    let initial_screen = screen.expect("No screen found");
    let screens = Screen::all().unwrap(); // is ordered by x by default
    let index = screens.iter().position(|s| s.display_info.id == initial_screen.display_info.id);
    let mut util_screens = Vec::new();
    if let Some(index) = index {
        util_screens = screens[index..].to_vec(); // take only the screens to the right of the initial screen
    }
    dbg!(util_screens.len());

    let mut screenshots = Vec::new(); // one Image for each involved screen
    let mut residual = selection.width();
    println!("SELECTION INFO: {:?}", selection);
    println!("initial residual: {}", residual);
    println!("----------------------------------");
    for mut screen in util_screens {

        // build the selection parameters for the i-th screen
        let y0 = selection.y0; // always correct
        let y1 = selection.y1; // always correct
        let x0;
        let x1;

        if selection.x0 > screen.display_info.width as f64 && selection.x1 > selection.x1 % screen.display_info.width as f64 { //
            dbg!("rescale both");
            x0 = selection.x0 % screen.display_info.width as f64;
            x1 = selection.x1 % screen.display_info.width as f64;
        } else if selection.x0 < screen.display_info.width as f64 && selection.x1 > selection.x1 % screen.display_info.width as f64 { //
            dbg!("second");
            x0 = selection.x0 % screen.display_info.width as f64;
            x1 = screen.display_info.width as f64;
        } else {
            dbg!("do not rescale");
            x0 = selection.x0;
            x1 = selection.x1;
        }

        dbg!(x0);
        dbg!(x1);
        let width =dbg! (((x1 - x0).abs()) as u32);
        let height =((y1 - y0).abs()) as u32;

        // Capture the screenshot using the adjusted coordinates on the unscaled screen
        let screen_shoot = screen.capture_area(x0 as i32, y0 as i32, width, height);
        dbg!(screen_shoot.is_ok());
        screenshots.push(screen_shoot.unwrap());

        residual =dbg! (residual - width as f64);

        if residual <= 0 as f64 { // more area to cover...
            println!("capture finished!");
            break;
        } println!("next screen...");


        // update selection parameters...
        selection.x0 = selection.x0 - width as f64;
        selection.x1 = selection.x1 - width as f64;

    }

    dbg!(screenshots.len());


    if screenshots.len() > 1 {
        // *************************************** merge all the screenshots ***************************************
        let image = from_multiple_image_to_single_image(screenshots);
        handle_save_screenshot(image);
    } else {
        // *************************************** save only one screenshot ***************************************
        let image = screenshots.pop().unwrap();
        handle_save_screenshot(image);
    }
}

pub fn capture_full_screen_screenshot (screen: Option<Screen>, all_screens: bool) {
    let screen_shoot;
    if all_screens {
        let screens = Screen::all().unwrap();
        let mut screenshots = Vec::new();
        for screen in screens {
            let screen_shoot = screen.capture();
            screenshots.push(screen_shoot.unwrap())
        }
        screen_shoot = from_multiple_image_to_single_image(screenshots);

    } else {
        let selected_screen = screen.expect("No screen selected");
        screen_shoot = selected_screen.capture().unwrap();
    }

    handle_save_screenshot(screen_shoot);
}

fn save_into_clipboard(output_path: &Path) -> Result<(), arboard::Error> {
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

fn handle_save_screenshot(screen_shoot: Image) {
    let path_str = Path::new("../config/config.txt");
    match read_config_file_savepath(path_str) {
        Ok(path) => {

            // check if the save path exists
            if !Path::new(&path).exists() {
                show_message_box("Error", "The path does not exist! Configure again the saving path from the settings.", MessageType::Error);
                panic!("The path does not exist!");
            }

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

            // convert into rgb8 and save into desired format
            if let Some(file_directory) = file_directory {
                let output_path = Path::new(&file_directory);
                let png_bytes = screen_shoot.to_png().expect("conversion error");
                let png_image = image::load_from_memory(&png_bytes).unwrap();
                let rgb_image: RgbImage = png_image.to_rgb8();
                let format = ImageFormat::from_path(output_path).expect("Conversion error");

                // save file into file system
                rgb_image.save_with_format(output_path, format).expect("conversion error");

                // save into the clipboard
                save_into_clipboard(output_path).expect("Error copying into the clipboard");

                // finish result
                show_message_box("Info", "Image successfully saved!", MessageType::Info)
            } else {
                show_message_box("Error", "Select a folder!", MessageType::Info);
            }

        },
        _ => {}
    }
}

fn show_message_box(title: &str, message: &str, mt: MessageType) {
    MessageDialog::new()
        .set_title(title)
        .set_text(message)
        .set_type(mt) // info , warning, error
        .show_alert()
        .expect("Failed to show the message box.");
}

fn from_multiple_image_to_single_image(images: Vec<Image>) -> Image {

    // convert into dynamic images
    let mut dynamic_images_screenshots = vec![];
    for image in images {
        //let image = image.unwrap();
        let png_bytes = image.to_png().unwrap();
        let png_image = image::load_from_memory(&png_bytes).unwrap().to_rgb8();
        dynamic_images_screenshots.push(png_image);
    }

    // set up all required conversion data from all the images
    let width_combined: u32 = dynamic_images_screenshots.iter().map(|image| image.width()).sum();
    let height_combined: u32 = dynamic_images_screenshots[0].height();
    let mut combined_image = RgbImage::new(width_combined, height_combined);

    // copy all the images horizontally in the dedicated buffer
    let mut current_x = 0;
    for image in &dynamic_images_screenshots {
        let width_image = image.width();
        for (x, y, pixel) in combined_image.enumerate_pixels_mut() {
            if x >= current_x && x < current_x + width_image {
                *pixel = image.get_pixel(x - current_x, y).clone();
            }
        }
        current_x += width_image;
    }
    let combined_image = DynamicImage::ImageRgb8(combined_image);
    let image = Image::new(combined_image.width(), combined_image.height(), combined_image.to_rgba8().into_raw());

    image
}