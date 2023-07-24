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

fn run_overlay() {
    
    let screens = Screen::all().unwrap();
    let screens_arc = Arc::new(screens);

    let (tx, rx): (mpsc::Sender<(Rect, Screen)>, mpsc::Receiver<(Rect, Screen)>) = mpsc::channel();

    let (width, height, leftmost, topmost) = compute_window_size();
    
    let overlay_window = WindowDesc::new(ScreenshotOverlay::new())
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
                Ok((selection, screen)) => {
                    capture_screenshot(selection, Some(screen));
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
