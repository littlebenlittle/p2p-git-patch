use super::{Commit, Repository as RepositoryTrait};

use std::path::PathBuf;

pub struct Repository {
    /// path to repository directory
    path: PathBuf,
}

impl TryFrom<PathBuf> for Repository {
    type Error = std::convert::Infallible;
    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        Ok(Self { path })
    }
}

impl RepositoryTrait for Repository {
    type CommitIter = BranchHistory;
    fn ancestor_iter(&self) -> Self::CommitIter {
        unimplemented!()
    }
    fn root(&self) -> Commit {
        unimplemented!()
    }
}

pub struct BranchHistory {}

impl Iterator for BranchHistory {
    type Item = Commit;
    fn next(&mut self) -> Option<Self::Item> {
        unimplemented!()
    }
}
