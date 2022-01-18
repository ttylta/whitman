use std::collections::HashMap;
use std::fs::{self, ReadDir};
use std::{fmt, path::Path};
use std::path::PathBuf;
use std::error::Error;

use orgize::Org;
use same_file::is_same_file;

use crate::config::{Env, Config};
use crate::constants::WHITMAN_CONFIG_FILE_NAME;
use crate::index::{Index, IndexRecord};
use crate::template::{TemplateMap, Template};
use crate::utils::get_file_vec;

#[derive(Debug, Clone)]
pub struct StateError;

impl fmt::Display for StateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "The current whitman state is invalid")
    }
}

impl Error for StateError {}

pub struct State<'a> {
    pub dir: PathBuf,
    pub env: String,
    pub index: Index<'a>,
    pub config: Option<Config>,
    pub templates: TemplateMap,
}

impl State<'_> {
    pub fn new(args: &[String]) -> State {
        let mut user_provided_dir = true;
        let current_dir = std::env::current_dir().unwrap();
        let dir_flag_index = args.iter().position(|r| r == "--dir").unwrap_or_else(|| { 
            user_provided_dir = false;
            0
        });

        if user_provided_dir && dir_flag_index + 1 > args.len() {
            user_provided_dir = false;
        }

        let mut dir: PathBuf = current_dir;
        if user_provided_dir {
            dir = Path::new(&args[dir_flag_index + 1]).to_path_buf();
        }

        let mut env: String = String::new();
        match std::env::var("WHITMAN_ENV") {
            Ok(val) => env = val,
            Err(_e) => env = "local".to_string(),
        }

        let mut config: Option<Config> = None;
        let reader: ReadDir;
        match fs::read_dir(&dir) {
            Ok(t) => reader = t,
            Err(_) => panic!("Invalid configuration directory."), 
        }

        for entry in reader {
            let entry = entry.expect("Unable to process file in provided directory.");
            let path = entry.path();

            let metadata = fs::metadata(&path).expect(
                "Error parsing file in the provided directory."
            );

            if metadata.is_file() && entry.file_name() == WHITMAN_CONFIG_FILE_NAME {
                let whitman_toml: String = fs::read_to_string(path).expect(
                    "The was an error parsing your configuration file."
                );
                config = Some(toml::from_str(&whitman_toml).unwrap());
            }
        }

        if config.is_none() {
            panic!("No configuration file found in the provided directory.");
        }

        State {
            env,
            dir,
            templates: HashMap::new(),
            config,
            index: Index {
                records: Vec::new(),
                org_map: HashMap::new(),
            },
        }
    }

    pub fn get_environment(&self) -> Result<&Env, Box<dyn Error>> {
        if self.config.is_none() {
            return Err(Box::new(StateError));
        }

        let config: &Config = self.config.as_ref().unwrap();
        if !config.environments.contains_key(&self.env) {
            return Err(Box::new(StateError));
        }

        let environment: &Env = config.environments.get(&self.env).unwrap();

        Ok(environment)
    }

    pub fn get_template_dir(&self) -> Result<PathBuf, Box<dyn Error>> {
        let environment: &Env = self.get_environment()?;

        let template_dir_path = self.dir.join(environment.template_dir.as_str());

        Ok(template_dir_path)
    }

    pub fn get_assets_dir(&self) -> Result<PathBuf, Box<dyn Error>> {
        let environment: &Env = self.get_environment()?;

        let assets_dir_path = self.dir.join(environment.assets_dir.as_str());

        Ok(assets_dir_path)
    }

    pub fn get_output_dir(&self) -> Result<PathBuf, Box<dyn Error>> {
        let environment: &Env = self.get_environment()?;

        let output_dir_path = self.dir.join(environment.output_dir.as_str());

        fs::create_dir_all(&output_dir_path)?;

        Ok(output_dir_path)
    }

    fn generate_templates(&mut self, dir: PathBuf) -> Result<(), Box<dyn Error>> {
        let walker = fs::read_dir(&dir).expect(
            "There was an error with the directory you provided."
        );

        println!("HERE");

        for entry in walker {
            let entry = entry?;    
            let buf = entry.path();
            let metadata = fs::metadata(&buf)?;

            let file_name: String;
            match entry.file_name().to_str() {
                Some(name) => file_name = String::from(name),
                None => continue,
            }
            
            if !metadata.is_file() && metadata.is_dir() {
                self.generate_templates(buf)
                    .expect("Unable to recursively walk through target directory.");
                continue;
            }

            if !file_name.contains(".hbs") {
                continue;
            }

            let template = Template {
                path: entry.path(),
                file_name: file_name.clone(),
                output: None,
            };
            
            println!("TEMPLATE");
            println!("{}", file_name);
            self.templates.insert(file_name, template);
        }

        Ok(())
    }

    fn generate_index(&mut self) -> Result<(), Box<dyn Error>> {
       let org_files = get_file_vec(&self.dir, &Some(String::from(".org")))?;
       let mut org_map: HashMap<String, Org> = HashMap::new();
       let mut records: Vec<IndexRecord> = Vec::new();

       for f in org_files.into_iter() {
        let output_dir = self.get_output_dir().unwrap();
           if  f.path.clone().starts_with(&output_dir) {
               continue;
           }

           let org = Org::parse_string(
                f.raw,
            );

            let mut template_name: String = String::new();
            for keyword in org.keywords() {
                if keyword.key.as_ref() == "WHITMAN_TEMPLATE" { template_name = keyword.value.to_string() }
            }

            org_map.insert(
                f.file_name.clone(), 
                org,
            );

            records.push(IndexRecord {
                template_name,
                file_name: f.file_name,
                path: f.path,
            });
       }

       
       self.index.records = records;
       self.index.org_map = org_map;

       Ok(())
    }

    pub fn init(&mut self) -> Result<(), Box<dyn Error>> {
        self.generate_templates(self.get_template_dir()?)?;
        self.generate_index()?;

        Ok(())
    }
}
