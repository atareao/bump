use serde::{
    Serialize,
    Deserialize
};
use std::path::PathBuf;
use tracing::{debug, error};

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

    pub async fn read(file_path: &PathBuf) -> Option<Self> {
        let content = match tokio::fs::read_to_string(file_path).await {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to read config file '{}': {}", file_path.display(), e);
                return None;
            }
        };
        match serde_yaml::from_str::<Self>(&content) {
            Ok(config) => Some(config),
            Err(e) => {
                error!("Failed to deserialize config file '{}': {}", file_path.display(), e);
                None
            }
        }
    }

    pub async fn write(&self, file: &PathBuf){
        match tokio::fs::write(
                file,
            serde_yaml::to_string(self).unwrap().as_bytes()).await{
            Ok(_) => debug!("Successfully wrote config file to {}", file.display()),
            Err(e) => error!("Failed to write config file: {}", e),
        }
    }
}
