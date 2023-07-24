use std::io::{self, BufRead, BufReader};
use std::fs::File;
use std::path::Path;

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