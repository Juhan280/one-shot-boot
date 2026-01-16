use std::{fs, io, path::PathBuf};

use crate::ENTRIES;

pub fn get_data_dir() -> PathBuf {
    let mut data_dir: PathBuf = std::env::var("LOCALAPPDATA")
        .expect("LOCALAPPDATA var doesn't exist")
        .into();
    data_dir.push(env!("CARGO_PKG_NAME"));

    data_dir
}

pub fn get_preferred_entry() -> Option<String> {
    let mut path = get_data_dir();
    path.push("preferred.txt");
    fs::read_to_string(path)
        .ok()
        .filter(|entry| ENTRIES.contains(entry))
}

pub fn set_preferred_entry(entry: &str) -> io::Result<()> {
    let mut path = get_data_dir();
    fs::create_dir_all(&path)?;
    path.push("preferred.txt");
    fs::write(path, entry)
}
