#![windows_subsystem = "windows"]

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
use utils::{compute_window_size, capture_screenshot, show_message_box, get_project_src_path};

const BROOM_ICON_DATA: &[u8] = include_bytes!("../../icons/broom.png");
const SAVE_ICON_DATA: &[u8] = include_bytes!("../../icons/save-icon.png");
const QUIT_ICON_DATA: &[u8] = include_bytes!("../../icons/back_from_overlay.png");

pub struct IconData {
    save_icon: Vec<u8>,  
    quit_icon: Vec<u8>,  
    broom_icon: Vec<u8>,
}

fn initialize_icons() -> anyhow::Result<IconData> {

    let broom_icon = load_from_memory(BROOM_ICON_DATA).context("Failed to load save icon")?;
    let save_icon = load_from_memory(SAVE_ICON_DATA).context("Failed to load quit icon")?;
    let quit_icon = load_from_memory(QUIT_ICON_DATA).context("Failed to load quit icon")?;


    Ok(IconData {
        save_icon: save_icon.to_rgb8().to_vec(),
        quit_icon: quit_icon.to_rgb8().to_vec(),
        broom_icon: broom_icon.to_rgb8().to_vec(),
    })
    
}

fn run_overlay(back: String) -> anyhow::Result<()> {
    
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
    let initial_state = AppState::new(screens_arc.clone(), Arc::new(Mutex::new(Some(tx))), back);
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
                    show_message_box("Info", "Image successfully saved!", Some(MessageType::Info));
                    let exe_path = get_project_src_path();
                    //questo percorso potrebbe rompersi su linux, sia per gli slash che per il .exe
                    let mut final_path;
                    if cfg!(windows){
                        final_path = exe_path.display().to_string() + r"/edit_gui/target/release/edit_gui.exe";
                    } else if cfg!(linux){
                        final_path = exe_path.display().to_string() + r"/edit_gui/target/release/edit_gui";
                    }
                    
                    let _ = Command::new(final_path)
                    .arg(&path)
                    .spawn()
                    .expect("Failed to start process");            
                }
                Err(err) => { show_message_box("Error", &err.to_string(), Some(MessageType::Error)) }
            }
        },
        Err(_) => {
            // Handle other possible errors here if needed.
            eprintln!("channel closed");
        }
    }
    
    Ok(())
}

fn main (){
    
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: overlay");
        std::process::exit(1);
    }
    let back = &args[1];
    println!("Received argument: {}", back);

    match run_overlay(back.clone()) {
        Ok(_) => {}
        Err(err) => { show_message_box("Error", &err.to_string(), Some(MessageType::Error)) }
    }
}
