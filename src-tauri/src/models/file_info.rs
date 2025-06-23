use serde::{Serialize, Deserialize};

#[derive(Serialize)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub size: u64,
}

#[derive(Serialize)]
pub struct FileGroup {
    pub key: String,
    pub files: Vec<FileInfo>,
}
