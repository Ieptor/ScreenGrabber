use std::fs::File;
use std::io::{self, BufRead, BufReader,  BufWriter, Write, Read};
use std::path::Path;


pub fn save_to_config_file(file_path: &Path, configuration: &str, type_: &str) -> io::Result<()> {
    // Read the content of the existing file into a string
    let mut file_content = String::new();
    let mut file = File::open(file_path)?;
    file.read_to_string(&mut file_content)?;

    // Find the position of the second line (if it exists) to preserve the rest of the file
    let second_line_start = file_content.find('\n').unwrap_or(file_content.len());

    // Create a temporary buffer to hold the modified content
    let mut temp_buf = Vec::new();

    // Modify the first line according to the type
    if type_ == "save_path" {
        temp_buf.extend_from_slice(configuration.as_bytes());
        // Append the rest of the content from the original file (beyond the second line)
        temp_buf.extend_from_slice(&file_content[second_line_start..].as_bytes());
    } else if type_ == "shortcut" {
        // Separate the content of the first line
        let first_line = file_content.lines().next().unwrap_or("");
        temp_buf.extend_from_slice(first_line.as_bytes());
        temp_buf.extend_from_slice(b"\n");
        // Append the modified configuration for the second line
        temp_buf.extend_from_slice(configuration.as_bytes());
    } else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid type parameter.",
        ));
    }

    // Reopen the file in write mode to truncate it
    let mut writer = BufWriter::new(File::create(file_path)?);
    writer.write_all(&temp_buf)?;

    Ok(())
}

pub fn read_config_file(file_path: &Path) -> io::Result<(String, String)> {
    let file = File::open(file_path)?;
    let path_reader = BufReader::new(&file);

    // Read the first line of the file (savepath)
    let savepath = match path_reader.lines().next() {
        Some(Ok(path)) => path,
        _ => "target".to_string(),
    };

    // Re-open the file using a new BufReader
    let file = File::open(file_path)?;
    let shortcut_reader = BufReader::new(&file);

    // Read the second line of the file (shortcut)
    let shortcut = match shortcut_reader.lines().nth(1) {
        Some(Ok(shortcut)) => shortcut,
        Some(Err(_err)) => {
            "ctrl + k".to_string()
        }
        None => {
            "ctrl + k".to_string()
        }
    };

    Ok((savepath, shortcut))
}

pub enum ShortcutValidation {
    Valid,
    Invalid,
    Incomplete,
}

pub fn validate_shortcut(shortcut: &String) -> ShortcutValidation {
    if shortcut.is_empty(){
        ShortcutValidation::Invalid
    } else if !shortcut.starts_with("ctrl +") {
        ShortcutValidation::Invalid
    } else {
        let remaining_chars = &shortcut["ctrl +".len()..];

        if remaining_chars.is_empty() {
            ShortcutValidation::Incomplete
        } else {
            ShortcutValidation::Valid
        }
    }
}
