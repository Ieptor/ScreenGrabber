//external dependencies
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, hotkey::{HotKey, Modifiers, Code}};
use winapi::um::winuser::{self, MSG};

//internal dependencies
mod utils;
use utils::read_config_file;

use std::process::Command;

pub fn parse_hotkey(shortcut_string: String) -> Option<(Modifiers, Code)> {

    let mut parts = shortcut_string.split('+').map(|s| s.trim().to_lowercase());
    
    let modifier = match parts.next().as_deref() {
        Some(control) => match control {
            "ctrl" => Modifiers::CONTROL,
            // add other modifiers as needed
            _ => Modifiers::CONTROL
        }
        _ => return None
    };

    
    let key = match parts.next().as_deref() {
        Some(key) => match key {
            "a" => Code::KeyA,
            "b" => Code::KeyB,
            "c" => Code::KeyC,
            "d" => Code::KeyD,
            "e" => Code::KeyE,
            "f" => Code::KeyF,
            "g" => Code::KeyG,
            "h" => Code::KeyH,
            "i" => Code::KeyI,
            "j" => Code::KeyJ,
            "k" => Code::KeyK,
            "l" => Code::KeyL,
            "m" => Code::KeyM,
            "n" => Code::KeyN,
            "o" => Code::KeyO,
            "p" => Code::KeyP,
            "q" => Code::KeyQ,
            "r" => Code::KeyR,
            "s" => Code::KeyS,
            "t" => Code::KeyT,
            "u" => Code::KeyU,
            "v" => Code::KeyV,
            "w" => Code::KeyW,
            "x" => Code::KeyX,
            "y" => Code::KeyY,
            "z" => Code::KeyZ,
            "0" => Code::Digit0,
            "1" => Code::Digit1,
            "2" => Code::Digit2,
            "3" => Code::Digit3,
            "4" => Code::Digit4,
            "5" => Code::Digit5,
            "6" => Code::Digit6,
            "7" => Code::Digit7,
            "8" => Code::Digit8,
            "9" => Code::Digit9,
            _ => Code::KeyK //default value
        }
        _ => return None
    };

    Some((modifier,key))

}


fn global_shortcut_handler(shortcut_command: Option<(Modifiers, Code)>) {
   
    if let Some((modifier, key)) = shortcut_command {
        let manager = GlobalHotKeyManager::new().unwrap();
        let hotkey = HotKey::new(Some(modifier),key);
        let _ = manager.register(hotkey);
        // Run the win32 event loop on the same thread
        unsafe{
            let mut msg: MSG = std::mem::zeroed();
            
            loop {
              
                if winuser::GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
                    winuser::TranslateMessage(&msg);
                    winuser::DispatchMessageW(&msg);

                    if let Ok(_event) = GlobalHotKeyEvent::receiver().try_recv() {
                        println!("sentito");
                        let _ = Command::new(r"..\overlay_process\release\overlay_process.exe")
                        .spawn()
                        .expect("Failed to start overlay process");
                    }
                }
                
            }
        }
    } else {
        eprintln!("Something wrong with shortcut command, not listening");
    }
}

pub fn main(){
    let config_file_path = std::path::Path::new("../config/config.txt");
    let mut shortcut_string = "ctrl + k".to_string(); //default value to be override

    match read_config_file(config_file_path) {
        Ok((_, shortcut)) => {
            shortcut_string = shortcut;
        },
        Err(_) => {
            eprintln!("Error reading config file");
        }
    }
    
    let shortcut_command = parse_hotkey(shortcut_string.clone());

    //let key_thread = std::thread::spawn(move || global_shortcut_handler(shortcut_command));
    //key_thread.join().expect("Failed to join the key-listening thread");

    global_shortcut_handler(shortcut_command);

}