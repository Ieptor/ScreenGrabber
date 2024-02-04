use std::fs::File;
use std::io::{self, BufRead, BufReader,  BufWriter, Write, Read};
use std::path::{Path};


pub fn save_to_config_file(file_path: &Path, configuration: &str, type_: &str) -> io::Result<()> {
    let mut file_content = String::new();
    let mut file = File::open(file_path)?;
    file.read_to_string(&mut file_content)?;

    let second_line_start = file_content.find('\n').unwrap_or(file_content.len());

    let mut temp_buf = Vec::new();

    if type_ == "save_path" {
        temp_buf.extend_from_slice(configuration.as_bytes());
        temp_buf.extend_from_slice(&file_content[second_line_start..].as_bytes());
    } else if type_ == "bg_shortcut" {
        let first_line = file_content.lines().next().unwrap_or("");
        temp_buf.extend_from_slice(first_line.as_bytes());
        temp_buf.extend_from_slice(b"\n");
        temp_buf.extend_from_slice(configuration.as_bytes());
    } else if type_ == "fs_shortcut" {
        let first_line = file_content.lines().next().unwrap_or("");
        let second_line = file_content.lines().nth(1).unwrap_or("");
        temp_buf.extend_from_slice(first_line.as_bytes());
        temp_buf.extend_from_slice(b"\n");
        temp_buf.extend_from_slice(second_line.as_bytes());
        temp_buf.extend_from_slice(b"\n");
        temp_buf.extend_from_slice(configuration.as_bytes());        
    } else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid type parameter.",
        ));
    }

    let mut writer = BufWriter::new(File::create(file_path)?);
    writer.write_all(&temp_buf)?;

    Ok(())
}

pub fn read_config_file(file_path: &Path) -> io::Result<(String, String, String)> {
    // Open the file for reading separately for each reader
    let file = File::open(file_path)?;

    // Read the first line of the file (savepath)
    let savepath = match BufReader::new(&file).lines().next() {
        Some(Ok(path)) => path,
        _ => "target".to_string(),
    };

    // Re-open the file using a new BufReader for the next line
    let file = File::open(file_path)?;
    let shortcut_reader = BufReader::new(&file);

    // Read the second line of the file (shortcut)
    let bg_shortcut = match shortcut_reader.lines().nth(1) {
        Some(Ok(bgshortcut)) => bgshortcut,
        Some(Err(_err)) => {
            "ctrl + k".to_string()
        }
        None => {
            "ctrl + k".to_string()
        }
    };

    // Re-open the file using a new BufReader for the third line
    let file = File::open(file_path)?;
    let shortcut_reader = BufReader::new(&file);

    let fs_shortcut = match shortcut_reader.lines().nth(2) {
        Some(Ok(fsshortcut)) => fsshortcut,
        Some(Err(_err)) => {
            "ctrl + f".to_string()
        }
        None => {
            "ctrl + f".to_string()
        }
    };

    Ok((savepath, bg_shortcut, fs_shortcut))
}


pub enum ShortcutValidation {
    Valid,
    Invalid,
    Incomplete,
    Same,
}

pub fn validate_shortcuts(bg_shortcut: &String, fs_shortcut: &String) -> ShortcutValidation {
    if bg_shortcut.is_empty() | fs_shortcut.is_empty() {
        ShortcutValidation::Invalid
    } else if bg_shortcut == fs_shortcut {
        ShortcutValidation::Same
    } else if !bg_shortcut.starts_with("ctrl +") | !fs_shortcut.starts_with("ctrl +") {
        ShortcutValidation::Invalid
    } else {
        let bg_remaining_chars = &bg_shortcut["ctrl +".len()..];
        let fs_remaining_chars = &fs_shortcut["ctrl +".len()..];

        if bg_remaining_chars.is_empty() ||  fs_remaining_chars.is_empty() {
            ShortcutValidation::Incomplete
        } else {
            ShortcutValidation::Valid
        }
    }
}

