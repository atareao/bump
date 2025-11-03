use serde::{
    Serialize,
    Deserialize
};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Replace {
    #[serde(default = "get_default_file")]
    pub file: String,
    #[serde(default = "get_default_search")]
    pub search: String,
    #[serde(default = "get_default_replace")]
    pub replace: String,
}

impl Replace {
    pub fn new(file: String, search: String, replace: String) -> Self {
        Replace {
            file,
            search,
            replace,
        }
    }
    pub fn default() -> Self {
        Self {
            file: get_default_file(),
            search: get_default_search(),
            replace: get_default_replace(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "get_default_current_version")]
    pub current_version: String,
    #[serde(default = "get_default_replaces")]
    pub replaces: Vec<Replace>,
}

fn get_default_current_version() -> String {
    "0.1.0".to_string()
}
fn get_default_file() -> String {
    "Cargo.toml".to_string()
}

fn get_default_search() -> String {
    "version = \"{{current_version}}\"".to_string()
}

fn get_default_replace() -> String {
    "version = \"{{new_version}}\"".to_string()
}

fn get_default_replaces() -> Vec<Replace> {
    vec![Replace::default()]
}

impl Config {
    pub fn new(current_version: String, replaces: Vec<Replace>) -> Self {
        Config {
            current_version,
            replaces,
        }
    }
    fn default() -> Self{
        Self{
            current_version: get_default_current_version(),
            replaces: get_default_replaces(),
        }
    }
    pub async fn write_default(file: &PathBuf){
        let default = Self::default();
        let _ = tokio::fs::write(
            file,
            serde_yaml::to_string(&default).unwrap().as_bytes()).await;
    }

    pub async fn read(file: &PathBuf) -> Option<Self>{
        match tokio::fs::read_to_string(file).await {
            Ok(content) => {
                match serde_yaml::from_str::<Config>(&content){
                    Ok(config) => Some(config),
                    Err(_) => None,
                }
            },
            Err(_) => None,
        }
    }
}
