use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Env {
    pub template_dir: String,
    pub output_dir: String,
    pub assets_dir: String,
}

#[derive(Deserialize)]
pub struct Config {
    pub environments: HashMap<String, Env>,
}
