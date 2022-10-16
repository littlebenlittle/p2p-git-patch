use super::Repository as RepositoryTrait;

pub struct Repository {}

impl Repository {
    pub fn new(path: std::path::PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        Ok( Self {})
    }
}

impl RepositoryTrait for Repository {}
