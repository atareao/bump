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
use tracing::{
    debug,
    error
};

mod config;
mod cli;
use clap::Parser;

use cli::{Cli, Commands};
use config::Config;

#[tokio::main]
async fn main() {
    let log_level = env::var("RUST_LOG").unwrap_or("DEBUG".to_string());
    tracing_subscriber::registry()
        .with(EnvFilter::from_str(&log_level).unwrap())
        .with(tracing_subscriber::fmt::layer())
        .init();

    debug!("log_level: {}", log_level);

    let cli = Cli::parse();
    match &cli.command {
        Commands::Upgrade(args) => {
            if args.change.is_none() {
                error!("Change type is required for upgrade command");
                return;
            }
            let config_path = get_config_path().await;

            match Config::read(&config_path).await {
                Some(mut config) => {
                    match increment_version(&config.current_version, args.change.as_ref().unwrap()) {
                        Ok(new_version) => {
                            println!("Current version: {}", config.current_version);
                            println!("New version (preview): {}", new_version);
                            for replace in &config.replaces {
                                let from = replace.search.replace("{{current_version}}", &config.current_version);
                                let to = replace.replace.replace("{{new_version}}", &new_version);
                                match replace_in_file(
                                        replace.file.as_str(), 
                                        &from, 
                                        &to).await {
                                    Ok(_) => debug!(UpdatedFile=%replace.file, "File updated successfully"),
                                    Err(e) => error!(File=%replace.file, "Failed to update file: {}", e),
                                }
                                let file_path = PathBuf::from(replace.file.as_str());
                                println!("Updating file: {}", file_path.display());
                            }
                            config.current_version = new_version;
                            config.write(&config_path).await;
                        },
                        Err(e) => {
                            error!("{}", e);
                        }
                    }
                },
                None => error!("Failed to read config file at {}", config_path.display()),
            }
        },
        Commands::Preview(args) => {
            let config_path = get_config_path().await;
            match Config::read(&config_path).await {
                Some(config) => {
                    match increment_version(&config.current_version, args.change.as_ref().unwrap()) {
                        Ok(new_version) => {
                            println!("Current version: {}", config.current_version);
                            println!("New version (preview): {}", new_version);
                        },
                        Err(e) => {
                            error!("{}", e);
                        }
                    }
                },
                None => error!("Failed to read config file at {}", config_path.display()),
            }
        },
        Commands::Show => {
            let config_path = get_config_path().await;
            match Config::read(&config_path).await {
                Some(config) => {
                    println!("Current version: {}", config.current_version);
                },
                None => error!("Failed to read config file at {}", config_path.display()),
            }
        },
    }
}

async fn get_config_path() -> PathBuf {
    match get_config().await{
        Some(path) => path,
        None => {
            let mut path = env::current_dir().unwrap();
            path.push("bump.yml");
            Config::write_default(&path).await;
            path
        }
    }
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

pub fn increment_version(current_version: &str, increment_type: &str) -> Result<String, String> {
    
    // 1. Parsear la versión actual (X.Y.Z)
    let parts: Vec<&str> = current_version.split('.').collect();

    // Verificación básica del formato: debe tener exactamente 3 partes.
    if parts.len() != 3 {
        return Err(format!("Formato de versión inválido. Se esperaba 'X.Y.Z', se recibió '{}'.", current_version));
    }

    // 2. Intentar parsear cada parte como un número entero (u32)
    let major = parts[0].parse::<u32>().map_err(|_| format!("Major (X) no es un número válido: {}", parts[0]))?;
    let minor = parts[1].parse::<u32>().map_err(|_| format!("Minor (Y) no es un número válido: {}", parts[1]))?;
    let patch = parts[2].parse::<u32>().map_err(|_| format!("Patch (Z) no es un número válido: {}", parts[2]))?;

    // 3. Aplicar el incremento basado en el tipo
    let new_version = match increment_type.to_lowercase().as_str() {
        "patch" => format!("{}.{}.{}", major, minor, patch + 1),
        
        "minor" => format!("{}.{}.{}", major, minor + 1, 0),
        
        "major" => format!("{}.{}.{}", major + 1, 0, 0),
        
        _ => return Err(format!("Tipo de incremento inválido. Se esperaba 'major', 'minor' o 'patch', se recibió '{}'.", increment_type)),
    };

    Ok(new_version)
}

/// Reemplaza todas las ocurrencias de 'from' por 'to' en el archivo especificado por 'path'.
///
/// # Argumentos
/// * `path` - La ruta del archivo (ej: "config.txt").
/// * `from` - La subcadena a buscar.
/// * `to` - La subcadena de reemplazo.
///
/// # Retorno
/// Devuelve un Result<(), std::io::Error> que indica éxito o un error de I/O.
pub async fn replace_in_file(path: &str, from: &str, to: &str) -> Result<(), std::io::Error> {
    
    // 1. Lectura Asíncrona: Lee todo el contenido del archivo en memoria.
    // El contenido se lee como un vector de bytes (Vec<u8>).
    let content_bytes = match tokio::fs::read(path).await {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("Error al leer el archivo '{}': {}", path, e);
            return Err(e);
        }
    };

    // 2. Conversión a String: Convierte los bytes leídos a una cadena UTF-8.
    let content = match String::from_utf8(content_bytes) {
        Ok(s) => s,
        Err(e) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("El archivo no contiene texto UTF-8 válido: {}", e),
            ));
        }
    };

    // 3. Reemplazo de la Cadena: Utiliza el método 'replace' de String.
    // Este paso es síncrono, ya que la sustitución se realiza en la memoria RAM.
    let modified_content = content.replace(from, to);

    // 4. Escritura Asíncrona: Sobrescribe el archivo original con el contenido modificado.
    match tokio::fs::write(path, modified_content).await {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("Error al escribir en el archivo '{}': {}", path, e);
            Err(e)
        }
    }
}
