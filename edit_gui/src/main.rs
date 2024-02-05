#![windows_subsystem = "windows"]

use druid::{AppLauncher, LocalizedString, WindowDesc};
use image::{load_from_memory, DynamicImage};
use std::fs;
use std::path::Path;
use druid::{Color};

mod edit;
use edit::*;

mod utils;
use utils::{resize_image};

mod drawing_tools;

pub struct ImageData {
    screenshot: DynamicImage,  
    icons: Vec<DynamicImage>,
    directory: String,
}

const SAVE_ICON_DATA: &[u8] = include_bytes!("../../icons/save64-icon.png");
const SHAPE_ICON_DATA: &[u8] = include_bytes!("../../icons/shapes-icon.png");
const TEXT_ICON_DATA: &[u8] = include_bytes!("../../icons/pencil.png");
const HIGHLIGHT_ICON_DATA: &[u8] = include_bytes!("../../icons/highlight-icon.png");
const BACK_ICON_DATA: &[u8] = include_bytes!("../../icons/back-icon.png");
const FORWARD_ICON_DATA: &[u8] = include_bytes!("../../icons/forward-icon.png");
const RESIZE_ICON_DATA: &[u8] = include_bytes!("../../icons/resize.png");
const RETURN_ICON_DATA: &[u8] = include_bytes!("../../icons/back.png");

const CIRCLE_ICON_DATA: &[u8] = include_bytes!("../../icons/circle.png");
const RECTANGLE_ICON_DATA: &[u8] = include_bytes!("../../icons/rectangle.png");
const TRIANGLE_ICON_DATA: &[u8] = include_bytes!("../../icons/triangle.png");

const ORANGE_ICON_DATA: &[u8] = include_bytes!("../../icons/arancione.png");
const GREEN_ICON_DATA: &[u8] = include_bytes!("../../icons/verde.png");
const YELLOW_ICON_DATA: &[u8] = include_bytes!("../../icons/giallo.png");

const SMALL_ICON_DATA: &[u8] = include_bytes!("../../icons/line-small.png");
const MEDIUM_ICON_DATA: &[u8] = include_bytes!("../../icons/line-medium.png");
const LARGE_ICON_DATA: &[u8] = include_bytes!("../../icons/line-large.png");

const CHECKBOX_ICON_DATA: &[u8] = include_bytes!("../../icons/checkbox.png");

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    
    if args.len() < 2 {
        eprintln!("Usage: edit_gui <path>");
        std::process::exit(1);
    }
    let path = &args[1];
    println!("Received argument: {}", path);
    
    let file_path = Path::new(path);

    let directory = file_path.parent().unwrap_or_else(|| {
        eprintln!("Invalid path provided.");
        std::process::exit(1);
    });
    let directory_str = directory.to_str().unwrap_or_else(|| {
        eprintln!("Invalid path provided.");
        std::process::exit(1);
    });

    
    let image_data = fs::read(path).expect("Error reading file");
    let dynamic_image = load_from_memory(&image_data).expect("failed to load image");
    let resized_image = resize_image(dynamic_image.clone(), (1200, 500));
    let icons_vec = initialize_icons();

    let imgstrct = ImageData{
        screenshot: resized_image, 
        icons: icons_vec,
        directory: directory_str.to_string(),
    };

    let main_window = WindowDesc::new(Edit::new(imgstrct))
        .window_size((1280.0, 790.0))
        .resizable(false)
        .title(LocalizedString::new("EDIT GUI"));

    // Create the initial app state
    let initial_state = AppState {};

    // Launch the application
    AppLauncher::with_window(main_window)
        .configure_env(|env, _| {
            // Set the background color of the window
            env.set(druid::theme::WINDOW_BACKGROUND_COLOR, Color::rgb(188.0/255.0, 189.0/255.0, 214.0/255.0));
        })
        .launch(initial_state)
        .expect("Failed to launch application");
}



fn initialize_icons() -> Vec<DynamicImage>{
    let save_icon = load_from_memory(SAVE_ICON_DATA).expect("Failed to load save icon");
    let shapes_icon = load_from_memory(SHAPE_ICON_DATA).expect("Failed to load shape icon");
    let text_icon = load_from_memory(TEXT_ICON_DATA).expect("Failed to load text icon");
    let highlight_icon = load_from_memory(HIGHLIGHT_ICON_DATA).expect("Failed to load highlight icon");
    let back_icon = load_from_memory(BACK_ICON_DATA).expect("Failed to load back icon");
    let forward_icon = load_from_memory(FORWARD_ICON_DATA).expect("Failed to load forward icon");
    let resize_icon = load_from_memory(RESIZE_ICON_DATA).expect("Failed to load save icon");
    let return_icon = load_from_memory(RETURN_ICON_DATA).expect("Failed to load forward icon");

    let circle_icon = load_from_memory(CIRCLE_ICON_DATA).expect("Failed to load forward icon");
    let rectangle_icon = load_from_memory(RECTANGLE_ICON_DATA).expect("Failed to load forward icon");
    let triangle_icon = load_from_memory(TRIANGLE_ICON_DATA).expect("Failed to load forward icon");

    let green_icon = load_from_memory(GREEN_ICON_DATA).expect("Failed to load forward icon");
    let orange_icon = load_from_memory(ORANGE_ICON_DATA).expect("Failed to load forward icon");
    let yellow_icon = load_from_memory(YELLOW_ICON_DATA).expect("Failed to load forward icon");

    let small_icon = load_from_memory(SMALL_ICON_DATA).expect("Failed to load forward icon");
    let medium_icon = load_from_memory(MEDIUM_ICON_DATA).expect("Failed to load forward icon");
    let large_icon = load_from_memory(LARGE_ICON_DATA).expect("Failed to load forward icon");

    let check_icon = load_from_memory(CHECKBOX_ICON_DATA).expect("Failed to load forward icon");

    vec![resize_icon, text_icon, highlight_icon, shapes_icon, back_icon, forward_icon, return_icon, circle_icon, rectangle_icon, triangle_icon, orange_icon, yellow_icon, green_icon, save_icon, check_icon, small_icon, medium_icon, large_icon]
}