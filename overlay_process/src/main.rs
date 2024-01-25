use druid::{AppLauncher, LocalizedString, WindowDesc, Size, Rect};
use screenshots::Screen;
use std::process::Command;
use std::sync::Arc;
use std::sync::mpsc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use anyhow::Context;
use native_dialog::MessageType;

mod overlay;
use overlay::*;
use image::load_from_memory;

mod utils;
use utils::{compute_window_size, capture_screenshot, show_message_box};

const SAVE_ICON_DATA: &[u8] = include_bytes!("../../icons/save-icon.png");
const QUIT_ICON_DATA: &[u8] = include_bytes!("../../icons/quit-icon.png");
const DELAY_ICON_DATA: &[u8] = include_bytes!("../../icons/delay-icon.png");

pub struct IconData {
    save_icon: Vec<u8>,  
    quit_icon: Vec<u8>,  
    delay_icon: Vec<u8>,
}

fn initialize_icons() -> anyhow::Result<IconData> {

    let save_icon = load_from_memory(SAVE_ICON_DATA).context("Failed to load save icon")?;
    let quit_icon= load_from_memory(QUIT_ICON_DATA).context("Failed to load quit icon")?;
    let delay_icon = load_from_memory(DELAY_ICON_DATA).context("Failed to load delay icon")?;

    Ok(IconData {
        save_icon: save_icon.to_rgb8().to_vec(),
        quit_icon: quit_icon.to_rgb8().to_vec(),
        delay_icon: delay_icon.to_rgb8().to_vec(),
    })
    
}

fn run_overlay() -> anyhow::Result<()> {
    
    let screens = Screen::all().context("Impossible to retrieve available screens.")?;
    let screens_arc = Arc::new(screens);
    let icon_data = initialize_icons()?;
    let (tx, rx): (mpsc::Sender<(Rect, Screen, i32)>, mpsc::Receiver<(Rect, Screen, i32)>) = mpsc::channel();
    let (width, height, leftmost, topmost) = compute_window_size()?;
    
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
        .context("Failed to launch application");


    thread::sleep(Duration::from_secs(1));
    match rx.recv() {
        Ok((selection, screen, _translation_factor)) => {
            //selection.x0 = selection.x0 - translation_factor.abs() as f64;
            //selection.x1 = selection.x1 - translation_factor.abs() as f64;
            match capture_screenshot(selection, Some(screen)) {
                Ok(path) => { 
                    show_message_box("Info", "Image successfully saved!", MessageType::Info);
                    let _ = Command::new(r"..\edit_gui\target\release\edit_gui.exe")
                    .arg(&path)
                    .spawn()
                    .expect("Failed to start process");            
                }
                Err(err) => { show_message_box("Error", &err.to_string(), MessageType::Error) }
            }
        },
        Err(_) => {
            // Handle other possible errors here if needed.
            println!("channel closed");
        }
    }
    println!("ending..");

    Ok(())

}

fn main (){
    match run_overlay() {
        Ok(_) => {}
        Err(err) => { show_message_box("Error", &err.to_string(), MessageType::Error) }
    }
}
