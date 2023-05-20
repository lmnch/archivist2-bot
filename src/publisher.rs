use std::path::Path;
use git2::{Oid, ObjectType, Commit, Direction, RemoteCallbacks, Tree};

use crate::config::Repository;

pub trait Publisher {
    /**
     * Add a file to the repository, commits it and pushs it to the server
     */
    fn publish_file(&self, repo: &Repository, added_file: &Path, message: &String) -> Result<Oid, git2::Error>;

    /**
     * Updates the filesystem
     */
    fn update_files(&self, repo: &Repository) -> Result<(), git2::Error>;
}


pub struct GitPublisher {
    ssh_key: String,
    branch: String
}

impl GitPublisher {
    pub fn new(ssh_key: String) -> GitPublisher {
        GitPublisher { ssh_key, branch: "master".to_string() }
    }

    fn find_last_commit<'a>(&'a self, repo: &'a git2::Repository) -> Result<Commit, git2::Error> {
        let obj = repo.head()?.resolve()?.peel(ObjectType::Commit)?;
        let last_commit = obj.into_commit().map_err(|_| git2::Error::from_str("Couldn't find commit"))?;
        println!("[repo: {}] Last commit: {}", repo.path().display(), last_commit.id());
        Ok(last_commit)
    }

    fn pull(&self, repo: &git2::Repository) -> Result<(), git2::Error> {
        repo.find_remote("origin")?.fetch(&[self.branch.clone()], None, None)?;
        let fetch_head = repo.find_reference("FETCH_HEAD")?;
        let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)?;
        let analysis = repo.merge_analysis(&[&fetch_commit])?;
        if analysis.0.is_up_to_date() {
            Ok(())
        } else if analysis.0.is_fast_forward() {
            let refname = format!("refs/heads/{}", self.branch);
            let mut reference = repo.find_reference(&refname)?;
            reference.set_target(fetch_commit.id(), "Fast-Forward")?;
            repo.set_head(&refname)?;
            repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
        } else {
            Err(git2::Error::from_str("Fast-forward only!"))
        }
    }

    fn push(&self, repo: &git2::Repository) -> Result<(), git2::Error> {
        let mut remote = repo.find_remote("origin")?;
        let mut remote_con = remote.connect_auth(Direction::Push, Some(self.get_remote_callback()), None)?;
        remote_con.remote().push(&["refs/heads/master:refs/heads/master"], None)?;
        println!("[repo: {}] Pushed", repo.path().display());
        Ok(())
    }

    fn get_remote_callback(&self) -> RemoteCallbacks {
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            git2::Cred::ssh_key(
                username_from_url.unwrap(),
                None,
                Path::new(&self.ssh_key),
                None
            )
        });

        callbacks
    }

    fn add_to_index<'a>(&'a self, repo: &'a git2::Repository, added_file: &Path) -> Result<Tree, git2::Error>{
        let mut index = repo.index()?;
        
        index.add_path(added_file)?;
        index.write_tree()?;
        let oid = index.write_tree()?;
        
        println!("[repo: {}] Added file {} to index", repo.path().display(), added_file.display());
        repo.find_tree(oid)
    }

    fn create_commit(&self, git_repo: &git2::Repository, sign: &git2::Signature, tree: &Tree, parent_commit: &Commit, message: &String)-> Result<Oid, git2::Error>{
        let commit_id = git_repo.commit(Some("HEAD"), &sign, &sign, message, &tree, &[&parent_commit])?;
        println!("[repo: {}] Created commit {}", git_repo.path().display(), commit_id);
        Ok(commit_id)
    }
}

impl Publisher for GitPublisher {
    fn publish_file(&self, repo: &Repository, added_file: &Path, message: &String) -> Result<Oid, git2::Error> {
        let git_repo = git2::Repository::open(repo.path())?;

        self.pull(&git_repo)?;

        let tree = self.add_to_index(&git_repo, added_file)?;       

        let parent_commit = self.find_last_commit(&git_repo)?;
        let commit_id = self.create_commit(&git_repo, &repo.signature()?, &tree, &parent_commit, message)?;

        self.push(&git_repo)?;

        Ok(commit_id)
    }

    fn update_files(&self, repo: &Repository) -> Result<(), git2::Error> {
         let git_repo = git2::Repository::open(repo.path())?;

        self.pull(&git_repo)?;

        Ok(())
    }
}
