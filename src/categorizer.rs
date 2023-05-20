use std::path::Path;
use serde_json::from_str;
use serde::{Serialize, Deserialize};

use crate::config::Repository;

pub struct CategorizationContext<'a> {
    repo: &'a Repository,
    chat_id: i64,
}

impl CategorizationContext<'_> {
    pub fn new(repo: &Repository, chat_id: i64) -> CategorizationContext {
        CategorizationContext { repo, chat_id }
    }
}

/**
 * Returns the path for a file based on a categorization string.
 */
pub trait Categorizer {
    fn categorize(&self, categorization: Option<&str>, context: CategorizationContext) -> String;
    fn tag(&mut self, tag: &str, path_matcher: &str, context: CategorizationContext);
}

pub struct ExactPathCategorizer {
}


impl Categorizer for ExactPathCategorizer {
    fn categorize(&self, categorization: Option<&str>, _context: CategorizationContext) -> String {
        categorization.unwrap_or("tmp.pdf").to_string()
    }

    fn tag(&mut self, _tag: &str, _path_matcher: &str, _context: CategorizationContext) {
        todo!()
    }
}


#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Category {
    tags: Vec<String>,
    path_matcher: String
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Categorization {
    default_category: String,
    categories: Vec<Category>
}

impl Category {
    fn matches(&self, tags: &Vec<String>) -> bool {
        for tag in &self.tags {
            if !tags.contains(&tag) {
                return false;
            }
        }
        return true;
    }
}

impl Categorization {
   fn get_category(&self, tags: &Vec<String>) -> Option<&Category> {
       for category in &self.categories {
           if category.matches(tags) {
               return Some(category);
           }
       }
       return None;
   } 
}


/**
 * Reads the categories directly from the repository.
 */
pub struct RepoBasedCategorizer {
}

impl RepoBasedCategorizer {
    pub fn new() -> RepoBasedCategorizer {
        RepoBasedCategorizer {  }
    }

    fn get_categories(&self, context: CategorizationContext) -> Option<Categorization> {
        let categorization_file = Path::new(context.repo.path()).join("categories.json");
        if categorization_file.exists() {
            let contents = std::fs::read_to_string(categorization_file).unwrap();
            let categories: Categorization = from_str(&contents).unwrap();
            return Some(categories);
        }
        return None;
    }
}

impl Categorizer for RepoBasedCategorizer {
    fn categorize(&self, categorization: Option<&str>, context: CategorizationContext) -> String {
        let categories = self.get_categories(context);
        if categories.is_none() {
            return "tmp.pdf".to_string();
        }
        if categorization.is_none() {
            return categories.unwrap().default_category.to_string();
        }

        let categorization_string = categorization.unwrap().to_string();
        let tags = categorization_string.split(" ");
        
        let found = categories.as_ref().unwrap().get_category(&tags.map(|s|s.to_string()).collect()).map(|cat| cat.path_matcher.clone());
        if found.is_none() {
            if categorization.is_some() {
                return categorization_string;
            }else{
                return categories.unwrap().default_category.to_string();
            }
        }
        return found.unwrap();
    }

    fn tag(&mut self, _tag: &str, _path_matcher: &str, _context: CategorizationContext) {
        todo!()
    }
}
