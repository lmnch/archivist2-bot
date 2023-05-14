use git2::Signature;


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
