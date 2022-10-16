mod eager;

pub use eager::Repository as EagerRepository;

pub trait Repository {
    type CommitIter: Iterator<Item = Commit>;
    /// return iter of ancestor from the HEAD of current branch
    fn ancestor_iter(&self) -> Self::CommitIter;
    /// return initial commit of the repository
    fn root(&self) -> Commit;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Commit {
}

impl Commit {
    pub fn is_in(&self, list: Vec<Commit>) -> bool {
        list.contains(self)
    }
    pub fn is_ancestor_of(&self, other: &Commit) -> bool {
        unimplemented!()
    }
}
