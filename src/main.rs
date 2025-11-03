use std::{env, path::PathBuf, str::FromStr};
use tracing::{debug, error};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

mod cli;
mod config;
use clap::Parser;

use cli::{Cli, Commands};
use config::Config;

const APP_NAME: &str = "vampus";

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
            // 1. Determinar el tipo de incremento usando la nueva lógica
            let increment_type = get_increment_type(args);

            let config_path = get_config_path().await;

            match Config::read(&config_path).await {
                Some(mut config) => {
                    // 2. Usar el increment_type
                    match increment_version(&config.current_version, increment_type) {
                        Ok(new_version) => {
                            println!("Current version: {}", config.current_version);
                            println!("New version (preview): {}", new_version);
                            for replace in &config.replaces {
                                let from = replace
                                    .search
                                    .replace("{{current_version}}", &config.current_version);
                                let to = replace.replace.replace("{{new_version}}", &new_version);
                                match replace_in_file(replace.file.as_str(), &from, &to).await {
                                    Ok(_) => {
                                        debug!(UpdatedFile=%replace.file, "File updated successfully")
                                    }
                                    Err(e) => {
                                        error!(File=%replace.file, "Failed to update file: {}", e)
                                    }
                                }
                                let file_path = PathBuf::from(replace.file.as_str());
                                println!("Updating file: {}", file_path.display());
                            }
                            config.current_version = new_version;
                            config.write(&config_path).await;
                        }
                        Err(e) => {
                            error!("{}", e);
                        }
                    }
                }
                None => error!("Failed to read config file at {}", config_path.display()),
            }
        }
        Commands::Preview(args) => {
            // 1. Determinar el tipo de incremento usando la nueva lógica
            let increment_type = get_increment_type(args);

            let config_path = get_config_path().await;
            match Config::read(&config_path).await {
                Some(config) => {
                    // 2. Usar el increment_type
                    match increment_version(&config.current_version, increment_type) {
                        Ok(new_version) => {
                            println!("Current version: {}", config.current_version);
                            println!("New version (preview): {}", new_version);
                        }
                        Err(e) => {
                            error!("{}", e);
                        }
                    }
                }
                None => error!("Failed to read config file at {}", config_path.display()),
            }
        }
        Commands::Show => {
            let config_path = get_config_path().await;
            match Config::read(&config_path).await {
                Some(config) => {
                    println!("Current version: {}", config.current_version);
                }
                None => error!("Failed to read config file at {}", config_path.display()),
            }
        }
    }
}

async fn get_config_path() -> PathBuf {
    let config_path = match std::env::current_dir() {
        Ok(mut path) => {
            path.push(format!(".{}.yml", APP_NAME));
            path
        }
        Err(e) => panic!(
            "Error fatal: No se puede obtener el directorio actual: {}",
            e
        ),
    };
    debug!(
        "Buscando archivo de configuración en: {}",
        config_path.display()
    );
    if tokio::fs::metadata(&config_path).await.is_err() {
        Config::write_default(&config_path).await;
        debug!("Archivo de configuración creado con valores por defecto.");
    }
    config_path
}

pub fn increment_version(current_version: &str, increment_type: &str) -> Result<String, String> {
    // 1. Parsear la versión actual (X.Y.Z)
    let parts: Vec<&str> = current_version.split('.').collect();

    // Verificación básica del formato: debe tener exactamente 3 partes.
    if parts.len() != 3 {
        return Err(format!(
            "Formato de versión inválido. Se esperaba 'X.Y.Z', se recibió '{}'.",
            current_version
        ));
    }

    // 2. Intentar parsear cada parte como un número entero (u32)
    let major = parts[0]
        .parse::<u32>()
        .map_err(|_| format!("Major (X) no es un número válido: {}", parts[0]))?;
    let minor = parts[1]
        .parse::<u32>()
        .map_err(|_| format!("Minor (Y) no es un número válido: {}", parts[1]))?;
    let patch = parts[2]
        .parse::<u32>()
        .map_err(|_| format!("Patch (Z) no es un número válido: {}", parts[2]))?;

    // 3. Aplicar el incremento basado en el tipo
    let new_version = match increment_type.to_lowercase().as_str() {
        "patch" => format!("{}.{}.{}", major, minor, patch + 1),

        "minor" => format!("{}.{}.{}", major, minor + 1, 0),

        "major" => format!("{}.{}.{}", major + 1, 0, 0),

        _ => {
            return Err(format!(
                "Tipo de incremento inválido. Se esperaba 'major', 'minor' o 'patch', se recibió '{}'.",
                increment_type
            ));
        }
    };

    Ok(new_version)
}

fn get_increment_type(args: &cli::UpgradeArgs) -> &'static str {
    if args.major {
        "major"
    } else if args.minor {
        "minor"
    } else {
        "patch"
    }
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
