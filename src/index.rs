use std::{path::PathBuf, collections::HashMap};
use std::error::Error;
use orgize::Org;

use crate::outline::Outline;
use crate::template::{Template, TemplateMap};

pub struct IndexRecord {
    pub file_name: String,
    pub template_name: String,
    pub path: PathBuf,
}

impl IndexRecord {
    pub fn outline<'a>(&self, org_map: &'a HashMap<String, Org<'a>>) -> Result<Outline<'a>, Box<dyn Error>> {
        let org = org_map.get(&self.file_name).unwrap().to_owned();
        Ok(Outline::from_org(org))
    }

    pub fn template<'a>(&self, template_map: &'a TemplateMap) -> Result<&'a Template, Box<dyn Error>> {
        let template = template_map.get(&self.template_name).unwrap_or_else(|| {
            panic!("Invalid template name for org file: {}", &self.template_name);
        });

        Ok(template)
    }
}

pub struct Index<'a> { 
    pub records: Vec<IndexRecord>,
    pub org_map: HashMap<String, Org<'a>>,
}
