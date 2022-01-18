use orgize::{Element, elements::Keyword, indextree::Arena};
use serde::Serialize;

use orgize::Org;

#[derive(Serialize)]
pub struct Tree<'a> {
    elements: Vec<&'a Element<'a>>,
}

#[derive(Serialize)]
pub struct Outline<'a> {
    pub title: String,
    pub keywords: Vec<Keyword<'a>>,
    pub org: &'a Org<'a>,
    pub tree: Tree<'a>,
}

fn collect_elements<'a>(arena: &'a Arena<Element<'a>>) -> Vec<&'a Element<'a>> {
    let mut elements: Vec<&Element<'a>> = Vec::new();

    for element in arena.iter() {
        elements.push(element.get());
    }

    elements
}

impl Outline<'_> {
    pub fn from_org<'a>(org: &'a Org) -> Outline<'a> {
        let mut title: String = String::new();
        for keyword in org.keywords() {
            if keyword.key.as_ref() == "TITLE" { title = keyword.value.to_string() }
        }

        let keywords = org.keywords().cloned().collect();
        let elements = collect_elements(org.arena());
        let tree = Tree { elements };

        Outline {
            title, 
            org,
            keywords, 
            tree, 
        }
    }
}
