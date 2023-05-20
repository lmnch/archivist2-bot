use regex::{Regex,Captures};
use crate::config::Repository;


#[derive(Debug)]
pub struct RuleContext<'a> {
    repo: &'a Repository,
    /**
     * Complete path splitted at path separator
     */
    path: &'a Vec<String>,
    /**
     * Index of the element in the path
     */
    index: usize
}

impl<'a> RuleContext<'a>{
    fn current(&self) -> &'a String {
        return &self.path[self.index];
    }

    fn until_current(&self) -> String {
        self.path[0..self.index].join("/")
    }

}

pub trait PathRule {
    fn resolve(&self, context: &RuleContext) -> String;
}

pub struct DefaultRule{}

impl PathRule for DefaultRule {
   fn resolve(&self, context: &RuleContext) -> String {
        context.current().clone()
   }
}

pub struct LatestRule<N: PathRule> {
    next: N
}

impl<T: PathRule> PathRule for LatestRule<T> {
    fn resolve(&self, context: &RuleContext) -> String {
       if context.current() == "^" {
            let paths = std::fs::read_dir(format!("{}/{}",context.repo.path(), context.until_current())).unwrap();

            return paths.map(|dir_entry| dir_entry.unwrap().file_name().into_string().unwrap()).max().unwrap();
       }
       // not resolvable by this rule
       return self.next.resolve(context);
    }

}


pub struct AddRule<N: PathRule> {
    next: N
}

impl<T: PathRule> PathRule for AddRule<T> {
    fn resolve(&self, context: &RuleContext) -> String {
        if context.current() == "+" {
            let regex =  Regex::new(r"^([A-Za-z]*)([0-9]+)?$").unwrap();

            let paths = std::fs::read_dir(format!("{}/{}",context.repo.path(), context.until_current())).unwrap();
            let dir_names : Vec<String> = paths.map(|dir_entry| dir_entry.unwrap().file_name().into_string().unwrap()).collect();

            let latest = dir_names.iter()
                .map(|dir| regex.captures(dir))
                .filter(|r|r.is_some())
                .map(|o|o.unwrap())
                .map(| r: Captures | { (r.get(1).map_or("".to_string(), |m| m.as_str().to_string()), r.get(2).map_or("".to_string(), |m| m.as_str().to_string()))})
                .map(| (prefix, index_str) | { (prefix, index_str.parse::<u32>().unwrap(), index_str)})
                .max_by(| (_a_prefix, a_index, _a_index_str), (_b_prefix, b_index, _b_index_str) | a_index.cmp(&b_index));
            if latest.is_none() {
                return "new".to_string();
            }

            let ( prefix, index, index_str) = latest.unwrap();

            // Count latest up by one
            let new_index = index + 1;

            let mut new_index_str : String = new_index.to_string();
            while new_index_str.len() < index_str.len() {
                new_index_str.insert(0, '0');
            }

            return prefix + new_index_str.as_str();
        }

        self.next.resolve(context)
    }
}



pub struct Matcher<T: PathRule> {
    rule_set: T
}

impl<T:PathRule> Matcher<T> {

    pub fn resolve(&self, repo: &Repository, path_matcher: String) -> String {
        let path: Vec<String> = path_matcher.split("/").map(|c| c.to_string()).collect();
        let mut resulting_path : Vec<String> = Vec::new();
        for i in 0..path.len() {
            let context = RuleContext{path: &path, index: i, repo};
            println!("[repo: {}] resolving context {:?}", repo.path(), &context);
            resulting_path.push(self.rule_set.resolve(&context))
        }

        return resulting_path.join("/");
    }
}


impl Matcher<AddRule<LatestRule<DefaultRule>>> {
    pub fn new() -> Matcher<AddRule<LatestRule<DefaultRule>>> {
       Matcher { rule_set: AddRule { next: LatestRule { next: DefaultRule{} } }}
    }
}
