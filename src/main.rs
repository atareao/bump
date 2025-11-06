use regex::Regex;
use std::{env, io, path::PathBuf, str::FromStr};
use tokio::fs;
use tracing::{debug, error};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

mod cli;
mod config;
use clap::Parser;
use cli::{Cli, Commands, VersionArgs}; // Asegúrate de importar VersionArgs
use config::Config;

const APP_NAME: &str = "vampus";

// =============================================================================================
// LÓGICA DE ARCHIVOS (TRANSACCIONAL)
// =============================================================================================

/// Simula el reemplazo con RegEx, verifica que el cambio se hizo, y devuelve el contenido modificado.
pub async fn simulate_replacement(
    path: &str,
    pattern_from: &str,
    replacement_to: &str,
    pattern_to: &str,
) -> Result<String, io::Error> {
    // 1. Compilar la expresión regular de búsqueda (FROM).
    let re_from = Regex::new(pattern_from).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Error al compilar RegEx FROM '{}': {}", pattern_from, e),
        )
    })?;

    // 2. Compilar la expresión regular de VERIFICACIÓN (TO).
    let re_to = Regex::new(pattern_to).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Error al compilar RegEx TO '{}': {}", pattern_to, e),
        )
    })?;

    // 3. Lectura y Conversión (I/O).
    let content_bytes = fs::read(path).await?;
    let content = String::from_utf8(content_bytes).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "El archivo '{}' no contiene texto UTF-8 válido: {}",
                path, e
            ),
        )
    })?;

    // 4. Verificación de existencia (CRÍTICO): El patrón antiguo DEBE estar presente.
    if !re_from.is_match(&content) {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "Patrón antiguo '{}' NO encontrado en '{}'.",
                pattern_from, path
            ),
        ));
    }

    // 5. Reemplazo de la Cadena usando la RegEx (Simulación).
    let modified_content = re_from.replace_all(&content, replacement_to);

    // 6. Verificación del Reemplazo (CRÍTICO): La nueva versión DEBE estar presente.
    if re_to.is_match(&modified_content) {
        Ok(modified_content.into_owned()) // Devuelve el String modificado
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "El patrón de nueva versión '{}' no se encontró después de la simulación. El reemplazo no coincidió con el formato esperado.",
                pattern_to
            ),
        ))
    }
}

/// Escribe el contenido pre-calculado en el archivo, reemplazando el contenido existente.
pub async fn apply_replacement(path: &str, content: &str) -> Result<(), io::Error> {
    fs::write(path, content.as_bytes()).await
}

// =============================================================================================
// ENUMS Y LÓGICA DE VERSIONES (SemVer)
// =============================================================================================

// NUEVA ENUM para manejar la operación (Incremento o Decremento)
enum Operation {
    Increment,
    Decrement,
}

/// Determina el tipo de cambio de versión basado en las flags mutuamente excluyentes.
fn get_version_change(args: &VersionArgs) -> (&'static str, Operation) {
    // Nota: Esta función ya no necesita el argumento 'operation' porque la operación
    // real se define en el match principal (Upgrade vs Downgrade).

    // Si la operación es explícita, el tipo de cambio se determina por la flag
    if args.major {
        ("major", Operation::Increment) // La operación de aquí es dummy, solo se usa el tipo de cambio
    } else if args.minor {
        ("minor", Operation::Increment)
    } else {
        ("patch", Operation::Increment) // Patch es el valor por defecto
    }
}

/// Lógica SemVer: Calcula la nueva (o anterior) versión.
fn calculate_version(
    current_version: &str,
    change_type: &str,
    operation: Operation,
) -> Result<String, String> {
    let parts: Vec<&str> = current_version.split('.').collect();
    if parts.len() != 3 {
        return Err(format!("Versión actual no válida: {}", current_version));
    }

    let mut major = parts[0]
        .parse::<i32>()
        .map_err(|_| "Error al parsear major".to_string())?;
    let mut minor = parts[1]
        .parse::<i32>()
        .map_err(|_| "Error al parsear minor".to_string())?;
    let mut patch = parts[2]
        .parse::<i32>()
        .map_err(|_| "Error al parsear patch".to_string())?;

    match operation {
        Operation::Increment => match change_type {
            "major" => {
                major += 1;
                minor = 0;
                patch = 0;
            }
            "minor" => {
                minor += 1;
                patch = 0;
            }
            "patch" => {
                patch += 1;
            }
            _ => return Err(format!("Tipo de cambio desconocido: {}", change_type)),
        },
        Operation::Decrement => match change_type {
            "major" => {
                if major == 0 {
                    return Err("No se puede hacer downgrade de major 0".to_string());
                }
                major -= 1;
                minor = 0;
                patch = 0;
            }
            "minor" => {
                if minor == 0 && major == 0 {
                    return Err("No se puede hacer downgrade de minor 0 y major 0".to_string());
                } else if minor == 0 {
                    return Err(
                        "No se puede hacer downgrade de minor 0 sin especificar major".to_string(),
                    );
                }
                minor -= 1;
                patch = 0;
            }
            "patch" => {
                if patch == 0 && minor == 0 && major == 0 {
                    return Err("No se puede hacer downgrade de 0.0.0".to_string());
                } else if patch == 0 {
                    return Err(
                        "No se puede hacer downgrade de patch 0 sin especificar minor".to_string(),
                    );
                }
                patch -= 1;
            }
            _ => return Err(format!("Tipo de cambio desconocido: {}", change_type)),
        },
    }

    if major < 0 || minor < 0 || patch < 0 {
        return Err("Resultado de versión inválido (negativo)".to_string());
    }

    Ok(format!("{}.{}.{}", major, minor, patch))
}

/// Obtiene la ruta del archivo de configuración.
async fn get_config_path() -> PathBuf {
    let mut config_path = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    config_path.push(format!(".{}.yml", APP_NAME));

    if !config_path.exists() {
        config::Config::write_default(&config_path).await;
    }
    config_path
}

// =============================================================================================
// MAIN Y LÓGICA DE COMANDOS
// =============================================================================================

#[tokio::main]
async fn main() {
    let log_level = env::var("RUST_LOG").unwrap_or("ERROR".to_string());
    tracing_subscriber::registry()
        .with(EnvFilter::from_str(&log_level).unwrap())
        .with(tracing_subscriber::fmt::layer())
        .init();

    debug!("log_level: {}", log_level);
    let cli = Cli::parse();

    if cli.debug {
        tracing_subscriber::registry()
            .with(EnvFilter::from_str("debug").unwrap())
            .with(tracing_subscriber::fmt::layer())
            .init();
        debug!("Modo DEBUG habilitado por flag CLI.");
    }

    match &cli.command {
        // -------------------------------------------------------------------------------------
        // COMANDO UPGRADE
        // -------------------------------------------------------------------------------------
        Commands::Upgrade(args) => {
            let (change_type, _) = get_version_change(args);
            let config_path = get_config_path().await;

            match Config::read(&config_path).await {
                Some(mut config) => {
                    match calculate_version(
                        &config.current_version,
                        change_type,
                        Operation::Increment,
                    ) {
                        Ok(new_version) => {
                            println!("Current version: {}", config.current_version);
                            println!("New version (preview): {}", new_version);

                            let mut modified_files = Vec::new();
                            let mut all_files_verified = true;

                            // FASE 1: VERIFICACIÓN Y SIMULACIÓN
                            println!("\n-- Verificando y simulando cambios... --");

                            for replace in &config.replaces {
                                let pattern_from = format!(
                                    "(?m){}",
                                    replace
                                        .search
                                        .replace("{{current_version}}", &config.current_version)
                                );

                                let pattern_to = format!(
                                    "(?m){}",
                                    replace.search.replace("{{current_version}}", &new_version)
                                );

                                let replacement_to =
                                    replace.replace.replace("{{new_version}}", &new_version);

                                // Corregido: Uso de macro debug! con un único string de formato
                                debug!(
                                    "Simulando archivo: {} | FROM: {} | TO (Verif): {}",
                                    replace.file, pattern_from, pattern_to
                                );

                                match simulate_replacement(
                                    replace.file.as_str(),
                                    &pattern_from,
                                    &replacement_to,
                                    &pattern_to,
                                )
                                .await
                                {
                                    Ok(content) => {
                                        modified_files.push((replace.file.clone(), content));
                                    }
                                    Err(e) => {
                                        error!(File=%replace.file, "FALLO DE SIMULACIÓN CRÍTICA: {}", e);
                                        all_files_verified = false;
                                        break;
                                    }
                                }
                            }

                            // FASE 2: EJECUCIÓN
                            if all_files_verified {
                                println!("\n-- Aplicando cambios a disco... --");

                                for (file_path, content) in modified_files {
                                    match apply_replacement(file_path.as_str(), &content).await {
                                        Ok(_) => {
                                            println!("✅ Actualizado: {}", file_path);
                                        }
                                        Err(e) => {
                                            error!(File=%file_path, "FALLO CRÍTICO al escribir: {}", e);
                                        }
                                    }
                                }

                                config.current_version = new_version;
                                config.write(&config_path).await;
                                println!(
                                    "\n🎉 Éxito: Versión de configuración actualizada a {}",
                                    config.current_version
                                );
                            } else {
                                error!(
                                    "\nUpgrade abortado. No se realizaron cambios en los archivos."
                                );
                            }
                        }
                        Err(e) => {
                            error!("Error al calcular la versión: {}", e);
                        }
                    }
                }
                None => error!("Failed to read config file at {}", config_path.display()),
            }
        }
        // -------------------------------------------------------------------------------------
        // COMANDO DOWNGRADE
        // -------------------------------------------------------------------------------------
        Commands::Downgrade(args) => {
            let (change_type, _) = get_version_change(args);
            let config_path = get_config_path().await;

            match Config::read(&config_path).await {
                Some(mut config) => {
                    match calculate_version(
                        &config.current_version,
                        change_type,
                        Operation::Decrement,
                    ) {
                        Ok(target_version) => {
                            println!("Current version: {}", config.current_version);
                            println!("Target downgrade version (preview): {}", target_version);

                            let current_version = config.current_version.clone();
                            let mut modified_files = Vec::new();
                            let mut all_files_verified = true;

                            // FASE 1: VERIFICACIÓN Y SIMULACIÓN
                            println!("\n-- Verificando y simulando cambios (Downgrade)... --");

                            for replace in &config.replaces {
                                let pattern_from = format!(
                                    "(?m){}",
                                    replace
                                        .search
                                        .replace("{{current_version}}", &current_version)
                                );

                                let pattern_to = format!(
                                    "(?m){}",
                                    replace
                                        .search
                                        .replace("{{current_version}}", &target_version)
                                );

                                let replacement_to =
                                    replace.replace.replace("{{new_version}}", &target_version);

                                // Corregido: Uso de macro debug! con un único string de formato
                                debug!(
                                    "Simulando archivo: {} | FROM: {} | TO (Verif): {}",
                                    replace.file, pattern_from, pattern_to
                                );

                                match simulate_replacement(
                                    replace.file.as_str(),
                                    &pattern_from,
                                    &replacement_to,
                                    &pattern_to,
                                )
                                .await
                                {
                                    Ok(content) => {
                                        modified_files.push((replace.file.clone(), content));
                                    }
                                    Err(e) => {
                                        error!(File=%replace.file, "FALLO DE SIMULACIÓN CRÍTICA: {}", e);
                                        all_files_verified = false;
                                        break;
                                    }
                                }
                            }

                            // FASE 2: EJECUCIÓN
                            if all_files_verified {
                                println!("\n-- Aplicando cambios a disco... --");

                                for (file_path, content) in modified_files {
                                    match apply_replacement(file_path.as_str(), &content).await {
                                        Ok(_) => {
                                            println!("✅ Actualizado (Downgrade): {}", file_path);
                                        }
                                        Err(e) => {
                                            error!(File=%file_path, "FALLO CRÍTICO al escribir: {}", e);
                                        }
                                    }
                                }

                                config.current_version = target_version.clone();
                                config.write(&config_path).await;
                                println!(
                                    "\n🎉 Éxito: Versión de configuración actualizada a {}",
                                    config.current_version
                                );
                            } else {
                                error!(
                                    "\nDowngrade abortado debido a fallos de simulación en los archivos."
                                );
                            }
                        }
                        Err(e) => {
                            error!("Error al calcular la versión de downgrade: {}", e);
                        }
                    }
                }
                None => error!("Failed to read config file at {}", config_path.display()),
            }
        }
        // -------------------------------------------------------------------------------------
        // COMANDO PREVIEW (Lógica corregida)
        // -------------------------------------------------------------------------------------
        Commands::Preview(args) => {
            let (change_type, _) = get_version_change(args);
            let config_path = get_config_path().await;

            // CORRECCIÓN: Evitamos llamar a to_string() en el enum.
            // 'Preview' se fija a 'Increment' por ser la expectativa por defecto.
            let operation = Operation::Increment;

            match Config::read(&config_path).await {
                Some(config) => {
                    match calculate_version(&config.current_version, change_type, operation) {
                        Ok(new_version) => {
                            println!("Current version: {}", config.current_version);
                            println!("Preview version (Increment): {}", new_version);
                        }
                        Err(e) => {
                            error!("Error al calcular la versión: {}", e);
                        }
                    }
                }
                None => error!("Failed to read config file at {}", config_path.display()),
            }
        }
        // -------------------------------------------------------------------------------------
        // COMANDO SHOW
        // -------------------------------------------------------------------------------------
        Commands::Show => {
            let config_path = get_config_path().await;
            match Config::read(&config_path).await {
                Some(config) => {
                    println!("Current project version: {}", config.current_version);
                }
                None => error!("Failed to read config file at {}", config_path.display()),
            }
        }
    }
}
