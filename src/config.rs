use std::collections::HashMap;
use std::fs;
use git2::Signature;

#[derive(Debug)]
pub struct Repository {
    path: String,
    secret: String,
    author_name: String,
    author_email: String
}

impl Repository {
    pub fn path(&self) -> &String {
        &self.path
    }
    
    pub fn signature(&self) -> Result<Signature, git2::Error> {
        Signature::now(self.author_name.clone().as_str(), self.author_email.clone().as_str())
    }

    pub fn new(path:String, secret: String, author_name: String, author_email: String) -> Repository {
        Repository { path, secret, author_name: author_name, author_email}
    }
}

pub trait RepositoryFactory{
   fn get_repositories(&self) -> Vec<&Repository>;
   fn get_repository(&self, secret: &String) -> Option<&Repository>;
}


unsafe impl Sync for EnvironmentRepositoryFactory {}

pub struct EnvironmentRepositoryFactory {
    pub repo: Repository
}

impl RepositoryFactory for EnvironmentRepositoryFactory {
    fn get_repositories(&self) -> Vec<&Repository> {
        return vec![&self.repo]
    }

    fn get_repository(&self, secret: &String) -> Option<&Repository> {
        // Only one repo can be configured via env consts
        let repo = self.get_repositories()[0];
        if &repo.secret == secret {
            return Some(&repo)
        }
        return None
    }
}

pub struct JsonRepositoryFactory {
    repos: HashMap<String, Repository>,
}

impl JsonRepositoryFactory {
   pub fn new(config_path: &str, author_name: &str, author_email: &str) -> JsonRepositoryFactory {
        let data = fs::read_to_string(config_path)
            .expect("Unable to read file");
        let repo_paths = serde_json::from_str::<HashMap<String, String>>(data.as_str()).unwrap();
        let mut repos = HashMap::new();
        for (secret, path) in repo_paths {
            repos.insert(secret.clone(), Repository::new(path, secret, author_name.to_string(), author_email.to_string()));
        }
        JsonRepositoryFactory { repos }
   } 
}

impl RepositoryFactory for JsonRepositoryFactory {
    fn get_repositories(&self) -> Vec<&Repository> {
        self.repos.values().collect()
    }

    fn get_repository(&self, secret: &String) -> Option<&Repository> {
        self.repos.get(secret)
    }
}



