use druid::Rect;
use screenshots::Screen;
use std::fs;

use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

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
        println!("Error opening config file: {:?}", err);
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
    
    if let Some(selected_screen) = screen {
        let scale_factor = selected_screen.display_info.scale_factor as f64;
        let unscaled_x0 = (selection.x0 - selected_screen.display_info.x as f64) / scale_factor;
        let unscaled_y0 = (selection.y0 - selected_screen.display_info.y as f64) / scale_factor;
        let _unscaled_x1 = (selection.x1 - selected_screen.display_info.x as f64) / scale_factor;
        let _unscaled_y1 = (selection.y1 - selected_screen.display_info.y as f64) / scale_factor;
        let unscaled_width = (selection.width() / scale_factor) as u32;
        let unscaled_height = (selection.height() / scale_factor) as u32;
        
        // Capture the screenshot using the adjusted coordinates on the unscaled screen
        let ss = selected_screen.capture_area(unscaled_x0 as i32, unscaled_y0 as i32, unscaled_width, unscaled_height);
        if let Err(err) = ss {
            eprintln!("Error capturing screenshot: {:?}", err);
        } else {
            let ss = ss.unwrap();
            let buffer = ss.to_png().unwrap();

            let path_str = std::path::Path::new("../config/config.txt");
            
            match read_config_file_savepath(path_str) {
                Ok(path) => {
                    let output_path = std::path::Path::new(&path).join("screenshot.png");
                    fs::write(output_path, buffer).unwrap();
                },
                //Default
                Err(_) => { fs::write("target/screenshot.png", buffer).unwrap();}
            }
        }

    } else {
        eprintln!("No screen selected.");
    }
}
