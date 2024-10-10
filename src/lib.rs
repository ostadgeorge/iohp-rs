use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use sha2::Digest;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Item {
    pub document_counter: String,
    pub title: String,
    pub metadata: HashMap<String, String>,
    pub sound_urls: Option<Vec<String>>,
}

pub fn file_name_by_url(url: &str, ext: &str) -> String {
    let hash = sha2::Sha256::digest(url.as_bytes());
    format!("{:x}.{}", hash, ext)
}