use std::path::Path;

use git2::{Repository,  Oid, ObjectType, Commit, Direction};

use crate::config;


pub fn add_and_commit(repo: &config::Repository, added_file: &Path) -> Result<Oid, git2::Error> {
    let git_repo = Repository::open(repo.path())?;
    
    let mut index = git_repo.index()?;
    
    index.add_path(added_file)?;
    index.write_tree()?;
    let oid = index.write_tree()?;
    let tree = git_repo.find_tree(oid)?;

    let parent_commit = find_last_commit(&git_repo)?;

    let sign = repo.signature()?;
    let commit_id = git_repo.commit(Some("HEAD"), &sign, &sign, "Lol", &tree, &[&parent_commit])?;

    push(&git_repo)?;

    Ok(commit_id)
}

fn find_last_commit(repo: &Repository) -> Result<Commit, git2::Error> {
    let obj = repo.head()?.resolve()?.peel(ObjectType::Commit)?;
    obj.into_commit().map_err(|_| git2::Error::from_str("Couldn't find commit"))
}


fn push(repo: &Repository) -> Result<(), git2::Error> {
    let mut remote = repo.find_remote("origin")?;
    remote.connect(Direction::Push)?;
    remote.push(&["refs/heads/master:refs/heads/master"], None)
}