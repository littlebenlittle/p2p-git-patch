use super::Database as DatabaseTrait;

pub struct Database {}

impl Database {
    pub fn new(path: std::path::PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {})
    }
}

impl DatabaseTrait for Database {}
