use druid::{AppLauncher, LocalizedString, WindowDesc, Size, Rect};
use screenshots::Screen;

use std::sync::Arc;
use std::sync::mpsc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

mod overlay;
use overlay::*;

mod utils;
use utils::{compute_window_size, capture_screenshot};
use crate::utils::capture_full_screen_screenshot;

pub struct IconData {
    save_icon: Vec<u8>,  
    quit_icon: Vec<u8>,  
    boh_icon: Vec<u8>,   
}

fn initialize_icons() -> IconData {
    //gestire errori
    let save_icon_data = image::open("../icons/save-icon.png").expect("Failed to load save icon");
    let quit_icon_data = image::open("../icons/quit-icon.png").expect("Failed to load quit icon");
    let boh_icon_data = image::open("../icons/boh-icon.png").expect("Failed to load boh icon");

    let save_icon_data = save_icon_data.to_rgb8();
    let quit_icon_data = quit_icon_data.to_rgb8();
    let boh_icon_data = boh_icon_data.to_rgb8();

    IconData {
        save_icon: save_icon_data.to_vec(),
        quit_icon: quit_icon_data.to_vec(),
        boh_icon: boh_icon_data.to_vec(),
    }
    
}

fn run_overlay() {
    
    let screens = Screen::all().unwrap();

    let screens_arc = Arc::new(screens);
    let icon_data = initialize_icons();

    let (tx, rx): (mpsc::Sender<(Rect, Screen, i32)>, mpsc::Receiver<(Rect, Screen, i32)>) = mpsc::channel();

    let (width, height, leftmost, topmost) = compute_window_size();
    
    let overlay_window = WindowDesc::new(ScreenshotOverlay::new(icon_data))
        .title(LocalizedString::new("Screenshot Overlay"))
        .transparent(true)
        .window_size(Size::new(width as f64, height as f64))
        .set_position((leftmost as f64, topmost as f64))
        .show_titlebar(false)
        .resizable(false);

    // Launch the overlay application
    let initial_state = AppState::new(screens_arc.clone(), Arc::new(Mutex::new(Some(tx))));
    let _overlay_state = AppLauncher::with_window(overlay_window)
        .launch(initial_state)
        .expect("Failed to launch application");


    thread::sleep(Duration::from_secs(1));
    match rx.recv() {
                Ok((mut selection, screen, translation_factor)) => {
                    //selection.x0 = selection.x0 - translation_factor.abs() as f64;
                    //selection.x1 = selection.x1 - translation_factor.abs() as f64;
                    capture_screenshot(selection, Some(screen), translation_factor);
                    //capture_full_screen_screenshot(Some(screen), false);
                    println!("ciao");
                },
                Err(_) => {
                    // Handle other possible errors here if needed.
                    println!("channel closed");
                }
            }

}

fn main (){
    run_overlay();
}
