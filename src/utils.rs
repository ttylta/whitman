use std::fs::{DirEntry, self};
use std::error::Error;
use std::path::{PathBuf, Path};

pub struct WalkerFile {
    pub file_name: String,
    pub path: PathBuf,
    pub raw: String,
}

pub fn get_file_vec(path: &Path, ext: &Option<String>) -> Result<Vec<WalkerFile>, Box<dyn Error>> {
    let mut entries: Vec<WalkerFile> = Vec::new();

    for entry in fs::read_dir(path).unwrap() {
        let entry = entry?;
        let path = entry.path();

        let metadata = fs::metadata(&path)?;

        if !metadata.is_file() && metadata.is_dir() {
            let mut internal_entries = get_file_vec(&path, ext)
                .expect("Unable to recursively walk through target directory.");
            entries.append(&mut internal_entries);
        }

        let file_name: String;
        match entry.file_name().to_str() {
            Some(name) => file_name = String::from(name),
            None => continue,
        }
        if ext.is_some() && !file_name.contains(ext.as_ref().unwrap().as_str()) {
            continue;
        }
        
        let raw = fs::read_to_string(&path).unwrap().to_owned();

        entries.push(WalkerFile {
            file_name,
            path,
            raw,
        });
    }

    Ok(entries)
}
