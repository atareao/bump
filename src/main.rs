use std::{
    env,
    path::PathBuf,
    str::FromStr
};
use tracing_subscriber::{
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter
};
use tracing::debug;

mod config;

use config::Config;

const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");


#[tokio::main]
async fn main() {
    let log_level = env::var("RUST_LOG").unwrap_or("DEBUG".to_string());
    tracing_subscriber::registry()
        .with(EnvFilter::from_str(&log_level).unwrap())
        .with(tracing_subscriber::fmt::layer())
        .init();

    debug!("log_level: {}", log_level);
}

async fn get_config() -> Option<PathBuf>{
    let mut current_path = std::env::current_dir().unwrap();
    current_path.push("bump.yml");
    debug!("Current path: {}", current_path.display());
    if(tokio::fs::metadata(&current_path)).await.is_ok(){
        return Some(current_path);
    }
    let mut exe_path = std::env::current_exe().unwrap();
    exe_path.push("bump.yml");
    debug!("Exe path: {}", exe_path.display());
    if(tokio::fs::metadata(&exe_path)).await.is_ok(){
        return Some(exe_path);
    }
    let mut home_path = dirs::home_dir().unwrap();
    debug!("Home path: {}", home_path.display());
    home_path.push(".bump.yml");
    if(tokio::fs::metadata(&home_path)).await.is_ok(){
        return Some(home_path);
    }
    let mut config_path = dirs::config_dir().unwrap();
    config_path.push("bump.yml");
    debug!("Config path: {}", config_path.display());
    if(tokio::fs::metadata(&config_path)).await.is_ok(){
        return Some(config_path);
    }
    let mut config_folder = dirs::config_dir().unwrap();
    config_folder.push("bump");
    config_folder.push("bump.yml");
    debug!("Config folder: {}", config_folder.display());
    if(tokio::fs::metadata(&config_folder)).await.is_ok(){
        return Some(config_folder);
    }
    None
}

