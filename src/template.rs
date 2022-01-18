use std::{path::PathBuf, collections::HashMap, io, fs};

pub struct Template {
    pub file_name: String,
    pub path: PathBuf,
    pub output: Option<String>,
}

impl Template {
    pub fn to_raw(&self) -> Result<String, io::Error> {
        fs::read_to_string(&self.path)
    }
}

pub type TemplateMap = HashMap<String, Template>;
