use std::borrow::Cow;
use druid::Rect;
use screenshots::Screen;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::path::PathBuf;
use image::{ImageFormat, RgbImage, GenericImageView};
use anyhow::{Result, anyhow};
use std::env;

extern crate arboard;
extern crate image;

use anyhow::{bail, Context};
use native_dialog::{FileDialog, MessageDialog, MessageType};
use screenshots::Image;

use std::process::{Command, Stdio};

pub fn compute_window_size()-> anyhow::Result<(i32, i32, i32, i32)> {
    let screens = Screen::all().context("Impossible to retrieve available screens.")?;

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

    Ok((width, height, leftmost, topmost))
}

pub fn capture_screenshot(mut selection: Rect, screen: Option<Screen>) -> Result<PathBuf> {

    // build a Vec<Screen> without the screens we are sure are not needed
    let initial_screen = screen.ok_or(anyhow!("No screen found"))?;
    let screens = Screen::all().context("Failed to get the list of screens")?;
    let index = screens
        .iter()
        .position(|s| s.display_info.id == initial_screen.display_info.id);
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
    for screen in util_screens {
        // build the selection parameters for the i-th screen
        let y0 = selection.y0; // always correct
        let y1 = selection.y1; // always correct
        let x0;
        let x1;

        if selection.x0 > screen.display_info.width as f64
            && selection.x1 > selection.x1 % screen.display_info.width as f64
        {
            dbg!("rescale both");
            x0 = selection.x0 % screen.display_info.width as f64;
            x1 = selection.x1 % screen.display_info.width as f64;
        } else if selection.x0 < screen.display_info.width as f64
            && selection.x1 > selection.x1 % screen.display_info.width as f64
        {
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
        let width = (x1 - x0).abs() as u32;
        let height = (y1 - y0).abs() as u32;

        // Capture the screenshot using the adjusted coordinates on the unscaled screen
        let screen_shoot = screen
            .capture_area(x0 as i32, y0 as i32, width, height)
            .context("Failed to capture screen")?;
        screenshots.push(screen_shoot);

        residual = dbg!(residual - width as f64);

        if residual <= 0 as f64 {
            // more area to cover...
            println!("capture finished!");
            break;
        }

        // update selection parameters...
        selection.x0 = selection.x0 - width as f64;
        selection.x1 = selection.x1 - width as f64;
    }

    dbg!(screenshots.len());

    if screenshots.len() > 1 {
        // *************************************** merge all the screenshots ***************************************
        let image = from_multiple_image_to_single_image(screenshots)?;
        handle_save_screenshot(image, false)
    } else {
        // *************************************** save only one screenshot ***************************************
        let image = screenshots.pop().ok_or(anyhow!("No screenshot available"))?;
        handle_save_screenshot(image, false)
    }

}


pub fn capture_full_screen_screenshot(screen: Option<Screen>, all_screens: bool, back: bool) -> Result<PathBuf> {
    let screen_shoot: Image;

    if all_screens {
        let screens = Screen::all().context("Failed to get the list of screens")?;
        let mut screenshots = Vec::new();

        for screen in screens {
            let screen_shoot = screen.capture().context("Failed to capture screen")?;
            screenshots.push(screen_shoot);
        }

        screen_shoot = from_multiple_image_to_single_image(screenshots)?;
    } else {
        let selected_screen = screen.ok_or(anyhow!("No screen selected"))?;
        screen_shoot = selected_screen.capture().context("Failed to capture screen")?;
    }

    handle_save_screenshot(screen_shoot, back)
    //Ok(())
}

fn read_config_file_savepath(file_path: &Path) -> anyhow::Result<String> {
    let file = File::open(file_path).with_context(|| format!("Failed to open config file {:?}", file_path))?;
    let reader = BufReader::new(file);

    if let Some(Ok(path)) = reader.lines().next() {
        if !path.is_empty() {
            return Ok(path);
        }
    }

    Err(anyhow::anyhow!("Config file is empty"))
} 

fn handle_save_screenshot(screen_shoot: Image, back: bool) -> Result<PathBuf> {

    let path_str = get_config_file_path();

    let path = match read_config_file_savepath(&path_str) {
        Ok(path) => path,
        Err(_) => {
            //show_message_box("Error", "The path does not exist! Configure the saving path from the settings.", Some(MessageType::Error));
            anyhow::bail!("The path does not exist! Configure the saving path from the settings.")
        }
    };

    // Verifica se il percorso di salvataggio esiste
    if !Path::new(&path).exists() {
        show_message_box("Error", "The path does not exist! Configure the saving path from the settings.", Some(MessageType::Error));
        anyhow::bail!("The path does not exist! Configure the saving path from the settings.");
    }

    // Seleziona la directory di salvataggio a partire da quella predefinita
    let file_directory = FileDialog::new()
        .add_filter("PNG", &["png"])
        .add_filter("JPG", &["jpg"])
        .add_filter("GIF", &["gif"])
        .add_filter("JPEG", &["jpeg"])
        .set_location(Path::new(&path))
        .set_filename("default")
        .show_save_single_file()
        .unwrap();
        //.context("Failed to show the save file dialog")?;

    // Converti in RGB8 e salva nel formato desiderato
    if let Some(file_directory) = file_directory {
        let output_path = Path::new(&file_directory);
        let png_bytes = screen_shoot.to_png()?;
        let png_image = image::load_from_memory(&png_bytes)?;
        let rgb_image: RgbImage = png_image.to_rgb8();

        //In windows it works, not in linux
        let format = ImageFormat::from_path(output_path)?;
        rgb_image.save_with_format(output_path, format)?;
        
        // Salva negli appunti
        save_into_clipboard(output_path).context("Error copying into the clipboard")?;

        return Ok(output_path.to_path_buf());

    } else {
        if back {
            show_message_box("Error", "Select a folder!", Some(MessageType::Info));
            let exe_path = get_project_src_path();
            let mut real_path = "".to_string();

            if cfg!(target_os = "windows"){
                real_path = exe_path.display().to_string() + r"/gui_sg/target/release/gui_sg.exe";
            }
            if cfg!(target_os = "linux"){
                real_path = exe_path.display().to_string() + r"/gui_sg/target/release/gui_sg";
            }

            let _ = Command::new(real_path)
                    .spawn()
                    .expect("Failed to start process");

            
        } 
        bail!("Select a folder!")
    }
}


pub fn save_into_clipboard(output_path: &Path) -> anyhow::Result<()> {
    let mut clipboard = arboard::Clipboard::new()
        .context("Failed to initialize clipboard")?;

    let image = image::open(output_path)
        .with_context(|| format!("Error opening the image at {:?}", output_path))?;

    let img = arboard::ImageData {
        width: image.width() as usize,
        height: image.height() as usize,
        bytes: Cow::from(image.to_rgba8().into_raw()),
    };

    if cfg!(target_os = "linux"){
        let output = Command::new("xclip")
        .arg("-selection")
        .arg("clipboard")
        .arg("-t")
        .arg("image/png") // Specify MIME type for PNG image
        .arg("-i")
        .arg(output_path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .expect("Failed to execute command");

        if output.status.success() {
            println!("Image copied to clipboard successfully");
        } else {
            println!("Failed to copy image to clipboard");
        }
    } else {
        clipboard.set_image(img)
        .with_context(|| "Error copying into the clipboard")?;
    }
        
    Ok(())
}

fn from_multiple_image_to_single_image(images: Vec<Image>) -> Result<Image> {

    // convert into dynamic images
    let mut dynamic_images_screenshots = vec![];
    for image in images {
        let png_bytes = image.to_png()?;
        let png_image = image::load_from_memory(&png_bytes)?;
        dynamic_images_screenshots.push(png_image.to_rgb8());
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

    let combined_image = image::DynamicImage::ImageRgb8(combined_image);
    let image = Image::new(combined_image.width(), combined_image.height(), combined_image.to_rgba8().into_raw());

    Ok(image)
}

pub fn show_message_box(title: &str, message: &str, mtt: Option<MessageType>) {
    let mt = mtt.unwrap_or(MessageType::Info);
    MessageDialog::new()
        .set_title(title)
        .set_text(message)
        .set_type(mt) // info , warning, error
        .show_alert()
        .expect("Failed to show the message box.");
}

pub fn get_config_file_path() -> PathBuf {
    let exe_dir = get_project_src_path();
    let relative_path = exe_dir.join("config").join("config.txt");
    relative_path
}

pub fn get_project_src_path() -> PathBuf {
    let exe_path = env::current_exe().expect("Failed to get current executable path");

    let mut exe_dir = exe_path.parent().expect("Failed to get parent directory");

    for _ in 0..3 {
        exe_dir = exe_dir.parent().expect("Failed to get parent directory");
    }

    exe_dir.to_path_buf()
}
