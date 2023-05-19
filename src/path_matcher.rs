
use crate::config::Repository;


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




pub struct Matcher<T: PathRule> {
    rule_set: T
}

impl Matcher<LatestRule<DefaultRule>> {
    pub fn new() -> Matcher<LatestRule<DefaultRule>> {
       Matcher { rule_set: LatestRule { next: DefaultRule{} } }
    }

    pub fn resolve(&self, repo: &Repository, path_matcher: String) -> String {
        let path: Vec<String> = path_matcher.split("/").map(|c| c.to_string()).collect();
        let mut resulting_path : Vec<String> = Vec::new();
        for i in 0..path.len() {
            let context = RuleContext{path: &path, index: i, repo};
            resulting_path.push(self.rule_set.resolve(&context))
        }

        return resulting_path.join("/");
    }
}


