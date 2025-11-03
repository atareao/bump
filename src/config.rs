use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Replace {
    pub file: String,
    pub search: String,
    pub replace: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub current_version: String,
    pub replaces: Vec<Replace>,
}
