use regex::Regex;
use std::{env, io, path::PathBuf};
use tokio::fs;
use tracing::debug;

use crate::config;
use crate::cli::VersionArgs;

const APP_NAME: &str = "vampus";


// =============================================================================================
// LÓGICA DE UTILIDAD
// =============================================================================================

/// Envuelve el patrón de búsqueda del usuario con grupos de captura () alrededor 
/// del texto que precede y sigue al marcador {{current_version}}.
/// 
/// Ejemplo: "^version = \"{{current_version}}\"$" -> "(^version = \"){{current_version}}(\"$)"
pub fn wrap_search_pattern(search_pattern: &str) -> String {
    // Intentar dividir la cadena usando el marcador de versión
    let parts: Vec<&str> = search_pattern.split("{{current_version}}").collect();

    // Verificamos que se haya dividido en al menos dos partes (antes y después del marcador)
    if parts.len() < 2 {
        // Si no se encuentra el marcador, se devuelve el patrón original.
        // Esto asume que el usuario sabe lo que hace, pero fallará si no hay versión.
        return search_pattern.to_string(); 
    }

    let prefix = parts[0];
    let suffix = parts[1];
    
    // Envolver el prefijo y el sufijo en grupos de captura de RegEx (usando el formato de reemplazo $1 y $2)
    format!("({prefix}){{{{current_version}}}}({suffix})")
}

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
            format!("Error compiling RegEx FROM '{}': {}", pattern_from, e),
        )
    })?;

    // 2. Compilar la expresión regular de VERIFICACIÓN (TO).
    let re_to = Regex::new(pattern_to).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Error compiling RegEx TO '{}': {}", pattern_to, e),
        )
    })?;

    // 3. Lectura y Conversión (I/O).
    let content_bytes = fs::read(path).await?;
    let content = String::from_utf8(content_bytes).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "File '{}' does NOT contain valid UTF-8 text: {}",
                path, e
            ),
        )
    })?;

    // 4. Verificación de existencia (CRÍTICO): El patrón antiguo DEBE estar presente.
    if !re_from.is_match(&content) {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "Last pattern '{}' NOT found in '{}'.",
                pattern_from, path
            ),
        ));
    }

    // 5. Reemplazo de la Cadena usando la RegEx (Simulación).
    let modified_content = re_from.replace_all(&content, replacement_to);
    debug!("Content modified simulated:\n{}", modified_content);

    // 6. Verificación del Reemplazo (CRÍTICO): La nueva versión DEBE estar presente.
    if re_to.is_match(&modified_content) {
        Ok(modified_content.into_owned()) // Devuelve el String modificado
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "New version pattern '{}' not found after simulation. Replacement did not match the expected format.",
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
pub enum Operation {
    Increment,
    Decrement,
}

/// Determina el tipo de cambio de versión basado en las flags mutuamente excluyentes.
pub fn get_version_change(args: &VersionArgs) -> (&'static str, Operation) {
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
pub fn calculate_version(
    current_version: &str,
    change_type: &str,
    operation: Operation,
) -> Result<String, String> {
    let parts: Vec<&str> = current_version.split('.').collect();
    if parts.len() != 3 {
        return Err(format!("Invalid current version format: {}", current_version));
    }

    let mut major = parts[0]
        .parse::<i32>()
        .map_err(|_| "Error parsing major component".to_string())?;
    let mut minor = parts[1]
        .parse::<i32>()
        .map_err(|_| "Error parsing minor component".to_string())?;
    let mut patch = parts[2]
        .parse::<i32>()
        .map_err(|_| "Error parsing patch component".to_string())?;

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
            _ => return Err(format!("Unknown change type: {}", change_type)),
        },
        Operation::Decrement => match change_type {
            "major" => {
                if major == 0 {
                    return Err("Cannot downgrade major version 0".to_string());
                }
                major -= 1;
                minor = 0;
                patch = 0;
            }
            "minor" => {
                if minor == 0 && major == 0 {
                    return Err("Cannot downgrade minor 0 when major is 0".to_string());
                } else if minor == 0 {
                    return Err(
                        "Cannot downgrade minor 0 without explicitly specifying --major".to_string(),
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
                        "Cannot downgrade patch 0 without explicitly specifying --minor or --major".to_string(),
                    );
                }
                patch -= 1;
            }
            _ => return Err(format!("Unknown change type: {}", change_type)),
        },
    }

    if major < 0 || minor < 0 || patch < 0 {
        return Err("Invalid (negative) version result".to_string());
    }

    Ok(format!("{}.{}.{}", major, minor, patch))
}

/// Obtiene la ruta del archivo de configuración.
pub async fn get_config_path() -> PathBuf {
    let mut config_path = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    config_path.push(format!(".{}.yml", APP_NAME));

    if !config_path.exists() {
        config::Config::write_default(&config_path).await;
    }
    config_path
}
