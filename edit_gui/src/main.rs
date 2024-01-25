use druid::{AppLauncher, LocalizedString, WindowDesc};
use image::{load_from_memory, DynamicImage, GenericImageView};
use std::fs;

mod edit;
use edit::*;

mod utils;
use utils::{resize_image};

pub struct ImageData {
    screenshot: DynamicImage,  
    icons: Vec<DynamicImage>
}

const SAVE_ICON_DATA: &[u8] = include_bytes!("../../icons/save64-icon.png");
const SHAPE_ICON_DATA: &[u8] = include_bytes!("../../icons/shapes-icon.png");
const TEXT_ICON_DATA: &[u8] = include_bytes!("../../icons/text-icon.png");
const HIGHLIGHT_ICON_DATA: &[u8] = include_bytes!("../../icons/highlight-icon.png");
const BACK_ICON_DATA: &[u8] = include_bytes!("../../icons/back-icon.png");
const FORWARD_ICON_DATA: &[u8] = include_bytes!("../../icons/forward-icon.png");

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    /*
    if args.len() < 2 {
        eprintln!("Usage: edit_gui <path>");
        std::process::exit(1);
    }
    let path = &args[1];
    println!("Received argument: {}", path);*/
    let path = r"C:\Users\pganc\Desktop\DAE.png";

    let image_data = fs::read(path).expect("Error reading file");
    let dynamic_image = load_from_memory(&image_data).expect("failed to load image");
    let resized_image = resize_image(dynamic_image.clone(), (1200, 500));

    let save_icon = load_from_memory(SAVE_ICON_DATA).expect("Failed to load save icon");
    let shapes_icon = load_from_memory(SHAPE_ICON_DATA).expect("Failed to load shape icon");
    let text_icon = load_from_memory(TEXT_ICON_DATA).expect("Failed to load text icon");
    let highlight_icon = load_from_memory(HIGHLIGHT_ICON_DATA).expect("Failed to load highlight icon");
    let back_icon = load_from_memory(BACK_ICON_DATA).expect("Failed to load back icon");
    let forward_icon = load_from_memory(FORWARD_ICON_DATA).expect("Failed to load forward icon");

    let imgstrct = ImageData{
        screenshot: resized_image, 
        icons: vec![save_icon, text_icon, highlight_icon, shapes_icon, back_icon, forward_icon]
        //text: text_icon,
        //shapes: shape_icon,
        //save: save_icon
    };

    let main_window = WindowDesc::new(Edit::new(imgstrct))
        .window_size((1280.0, 720.0))
        .resizable(false)
        .title(LocalizedString::new("EDIT GUI"));

    // Create the initial app state
    let initial_state = AppState {};

    // Launch the application
    AppLauncher::with_window(main_window)
        .launch(initial_state)
        .expect("Failed to launch application");
}


