


/**
 * Returns the path for a file based on a categorization string.
 */
pub trait Categorizer {
    fn categorize(&self, categorization: Option<&str>) -> String;
    fn tag(&mut self, tag: &str, path_matcher: &str);
}

pub struct ExactPathCategorizer {
}

impl ExactPathCategorizer {
    pub fn new() -> ExactPathCategorizer {
        ExactPathCategorizer {  }
    }
}

impl Categorizer for ExactPathCategorizer {
    fn categorize(&self, categorization: Option<&str>) -> String {
        categorization.unwrap_or("tmp.pdf").to_string()
    }

    fn tag(&mut self, _tag: &str, _path_matcher: &str) {
        todo!()
    }
}


