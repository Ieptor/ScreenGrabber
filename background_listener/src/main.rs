#![windows_subsystem = "windows"]

//external dependencies
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, hotkey::{HotKey, Modifiers, Code}};
use winapi::um::winuser::{self, MSG};
use winapi::shared::winerror::WAIT_TIMEOUT;
use winapi::um::winbase::WAIT_OBJECT_0;

//internal dependencies
mod utils;
use utils::read_config_file;
use screenshots::Screen;
use std::process::Command;
use overlay_process::utils::{capture_full_screen_screenshot, get_config_file_path, get_project_src_path, show_message_box};

extern crate systray;

use std::sync::{Arc, Mutex};
use std::thread;
use systray::{Application};

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



pub fn main(){   
    let config_path = get_config_file_path();
    let mut shortcut_string = "ctrl + k".to_string(); //default value to be override
    let mut shortcut_fs = "ctrl + f".to_string();

    match read_config_file(&config_path) {
        Ok((_, shortcut, fs)) => {
            shortcut_string = shortcut;
            shortcut_fs = fs
        },
        Err(_) => {
            eprintln!("Error reading config file");
        }
    }

    let helper_str = format!(
        "The program is now listening in the background.\n\
        Click ({}) to open the screenshot overlay\n\
        Blick ({}) to take a fullscreen screenshot.",
        shortcut_string,
        shortcut_fs
    );
    show_message_box("Background listener", &helper_str, None);

    let shortcut_command = parse_hotkey(shortcut_string.clone());
    let shortcut_fs_command = parse_hotkey(shortcut_fs.clone());

    // put app in systray
    let mut app = Application::new().unwrap();

    let icon_path = get_project_src_path();
    let final_path = icon_path.display().to_string() + r"\background_listener\src\icon.ico";
     // Set icon
    app.set_icon_from_file(&final_path).unwrap();

    // Add a quit item to the menu
    app.add_menu_item("Quit", |window| {
        window.quit();
        Ok::<(), systray::Error>(()) // Specify the error type explicitly
    }).unwrap();


    //needed to notify closing in systray to actual listener closing
    let running = Arc::new(Mutex::new(true));
    let running_clone = Arc::clone(&running);

    let handle = thread::spawn(move || {
        loop {
            if let Ok(event) = app.wait_for_message() {
                match event {
                    () => {
                        //app has been closed, remove listener
                        *running_clone.lock().unwrap() = false; // Use cloned Arc inside the closure
                        break;
                    },
                }
            }
        }
    });

    if let Some((modifier, key1)) = shortcut_command {
        if let Some((modifier2, key2)) = shortcut_fs_command {
            let manager = GlobalHotKeyManager::new().unwrap();
            let hotkey1 = HotKey::new(Some(modifier),key1);
            let hotkey2 = HotKey::new(Some(modifier2),key2);

            let id1 = hotkey1.id();
            let id2 = hotkey2.id();
            let _ = manager.register(hotkey1);
            let _2 = manager.register(hotkey2);

            //Run the win32 event loop on the same thread
            //Questo loop unsafe è specifico di windows, dentro poi chiama le funzionalità del global hotkey, controllando anche il mutex del system tray se è stata quittata l'app.
            #[cfg(target_os = "windows")] {
                unsafe{
                    let mut msg: MSG = std::mem::zeroed();
                    
                    loop {
                        // Check if we need to close
                        if !*running.lock().unwrap() {
                            break;
                        }
                
                        // Check for messages or wait for a timeout
                        let result = winuser::MsgWaitForMultipleObjectsEx(0, std::ptr::null(), 0, winuser::QS_ALLINPUT, winuser::MWMO_INPUTAVAILABLE);
                
                        if result == WAIT_OBJECT_0 {
                            // There are messages to process
                            while winuser::PeekMessageW(&mut msg, std::ptr::null_mut(), 0, 0, winuser::PM_REMOVE) != 0 {
                                winuser::TranslateMessage(&msg);
                                winuser::DispatchMessageW(&msg);
                            }
                        } else if result == WAIT_TIMEOUT {
                            // Check the mutex after a timeout
                            if !*running.lock().unwrap() {
                                break;
                            }
                        }
                
                        // Check for global hotkey events
                        if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
                            if event.id == id1 {
                                let exe_path = get_project_src_path();
                                //questo percorso potrebbe rompersi su linux, sia per gli slash che per il .exe
                                let final_path = exe_path.display().to_string() + r"\overlay_process\target\release\overlay_process.exe";
                                let _ = Command::new(final_path)
                                    .arg("f")
                                    .spawn()
                                    .expect("Failed to start overlay process");
                            } else if event.id == id2 {
                                let screens = Screen::all().unwrap();
                                match capture_full_screen_screenshot(Some(screens[0]), true) {
                                    Ok(path) => {
                                        let exe_path = get_project_src_path();
                                        //questo percorso potrebbe rompersi su linux, sia per gli slash che per il .exe
                                        let final_path = exe_path.display().to_string() + r"\edit_gui\target\release\edit_gui.exe";
                                        let _ = Command::new(final_path)
                                        .arg(&path)
                                        .spawn()
                                        .expect("Failed to start process");
                                    }
                                    Err(err) => {
                                        eprintln!("Failed to capture screenshot: {}", err);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        
           
            #[cfg(target_os = "linux")] {
                //creare qui un loop per linux
                //deve fare le stesse cose (loopare controllando il mutex se deve o no fare break e checkare il globalhotkeyevent)
                //esempio su linux nella doc: https://github.com/tauri-apps/global-hotkey/blob/dev/examples/tao.rs
            }
        }
    }
    handle.join().unwrap();
}