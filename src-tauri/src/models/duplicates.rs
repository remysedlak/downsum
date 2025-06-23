use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DuplicateFile {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub duplicate_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DuplicateGroup {
    pub original_name: String,
    pub files: Vec<DuplicateFile>,
    pub total_size: u64,
}
