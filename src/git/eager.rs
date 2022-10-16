use super::{Commit, Repository as RepositoryTrait};

pub struct Repository {}

impl Repository {
    pub fn new(path: std::path::PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {})
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

struct BranchHistory {}

impl Iterator for BranchHistory {
    type Item = Commit;
    fn next(&mut self) -> Option<Self::Item> {
        unimplemented!()
    }
}
