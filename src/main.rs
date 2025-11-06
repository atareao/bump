use std::{env, str::FromStr};
use tracing::{debug, error};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
use clap::Parser;

mod cli;
mod config;
mod utils;
use cli::{Cli, Commands}; // Asegúrate de importar VersionArgs
use config::Config;
use utils::{
    apply_replacement,
    simulate_replacement,
    get_config_path,
    calculate_version,
    Operation,
    get_version_change,
    wrap_search_pattern,
};

// =============================================================================================
// MAIN Y LÓGICA DE COMANDOS
// =============================================================================================

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    
    let log_filter_str = if cli.debug {
        "debug".to_string()
    } else {
        env::var("RUST_LOG").unwrap_or("ERROR".to_string())
    };

    // Inicialización del subscriber UNA SOLA VEZ
    tracing_subscriber::registry()
        .with(EnvFilter::from_str(&log_filter_str)
            .unwrap_or_else(|_| EnvFilter::from_str("error").unwrap())) 
        .with(tracing_subscriber::fmt::layer())
        .init();

    debug!("log_level: {}", log_filter_str);

    if cli.debug {
        debug!("DEBUG mode enabled via CLI flag.");
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
                            
                            // El string de reemplazo usa los grupos de captura $1 y $2.
                            let replacement_to = format!("${{1}}{}${{2}}", new_version);
                            debug!("Replacement TO string: {}", replacement_to);

                            let mut modified_files = Vec::new();
                            let mut all_files_verified = true;

                            // FASE 1: VERIFICACIÓN Y SIMULACIÓN
                            print!("-- Verifying and simulating changes... --");

                            for replace in &config.replaces {
                                
                                // APLICAR LÓGICA DE ENVOLTURA AUTOMÁTICA
                                let wrapped_search = wrap_search_pattern(replace.pattern.as_str());
                                debug!("Wrapped search pattern: {}", wrapped_search);
                                
                                // El patrón de búsqueda (FROM) usa la versión actual
                                let pattern_from = format!(
                                    "(?m){}",
                                    wrapped_search.replace(
                                        "{{current_version}}",
                                        &config.current_version.replace(".", "\\."))
                                );
                                debug!("Pattern FROM: {}", pattern_from);

                                // El patrón de verificación (TO) usa la nueva versión
                                let pattern_to = format!(
                                    "(?m){}",
                                    wrapped_search.replace(
                                        "{{current_version}}",
                                        &new_version.replace(".", "\\."))
                                );
                                debug!("Pattern TO: {}", pattern_to);

                                debug!(
                                    "Simulating file: {} | FROM: {} | TO (Verif): {}",
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
                                        error!(File=%replace.file, "CRITICAL SIMULATION FAILURE: {}", e);
                                        all_files_verified = false;
                                        break;
                                    }
                                }
                            }

                            // FASE 2: EJECUCIÓN (sin cambios)
                            if all_files_verified {
                                print!("-- Applying changes... --");

                                for (file_path, content) in modified_files {
                                    match apply_replacement(file_path.as_str(), &content).await {
                                        Ok(_) => {
                                            println!("✅ Updated: {}", file_path);
                                        }
                                        Err(e) => {
                                            error!(File=%file_path, "CRITICAL WRITE FAILURE: {}", e);
                                        }
                                    }
                                }

                                config.current_version = new_version;
                                config.write(&config_path).await;
                                println!(
                                    "🎉 Success: Config version updated to {}",
                                    config.current_version
                                );
                            } else {
                                error!(
                                    "Upgrade aborted. No changes were written to files."
                                );
                            }
                        }
                        Err(e) => {
                            error!("Error calculating the version: {}", e);
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
                            
                            // El string de reemplazo usa los grupos de captura $1 y $2.
                            let replacement_to = format!("${{1}}{}${{2}}", target_version);

                            let current_version = config.current_version.clone();
                            let mut modified_files = Vec::new();
                            let mut all_files_verified = true;

                            // FASE 1: VERIFICACIÓN Y SIMULACIÓN
                            println!("-- Verifying and simulating changes (Downgrade)... --");

                            for replace in &config.replaces {
                                
                                // APLICAR LÓGICA DE ENVOLTURA AUTOMÁTICA
                                let wrapped_search = wrap_search_pattern(replace.pattern.as_str());

                                // El patrón de búsqueda (FROM) usa la versión actual
                                let pattern_from = format!(
                                    "(?m){}",
                                    wrapped_search.replace(
                                        "{{current_version}}",
                                        &current_version.replace(".", "\\."))
                                );
                                debug!("Pattern FROM: {}", pattern_from);

                                // El patrón de verificación (TO) usa la versión de destino
                                debug!("Wrapped search pattern: {}", wrapped_search);
                                let pattern_to = format!(
                                    "(?m){}",
                                    wrapped_search.replace(
                                        "{{current_version}}",
                                        &target_version.replace(".", "\\."))
                                );
                                debug!("Pattern TO: {}", pattern_to);
                                
                                debug!(
                                    "Simulating file: {} | FROM: {} | TO (Verif): {}",
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
                                        error!(File=%replace.file, "CRITICAL FAILURE SIMULATION: {}", e);
                                        all_files_verified = false;
                                        break;
                                    }
                                }
                            }

                            // FASE 2: EJECUCIÓN (sin cambios)
                            if all_files_verified {
                                print!("-- Applying changes... --");

                                for (file_path, content) in modified_files {
                                    match apply_replacement(file_path.as_str(), &content).await {
                                        Ok(_) => {
                                            println!("✅ Updated (Downgrade): {}", file_path);
                                        }
                                        Err(e) => {
                                            error!(File=%file_path, "CRITICAL WRITE FAILURE: {}", e);
                                        }
                                    }
                                }

                                config.current_version = target_version.clone();
                                config.write(&config_path).await;
                                println!(
                                    "\n🎉 Success: Config version updated to {}",
                                    config.current_version
                                );
                            } else {
                                error!(
                                    "\nDowngrade aborted due to simulation failures in files."
                                );
                            }
                        }
                        Err(e) => {
                            error!("Error calculating the downgrade version: {}", e);
                        }
                    }
                }
                None => error!("Failed to read config file at {}", config_path.display()),
            }
        }
        // -------------------------------------------------------------------------------------
        // COMANDO PREVIEW (sin cambios)
        // -------------------------------------------------------------------------------------
        Commands::Preview(args) => {
            let (change_type, _) = get_version_change(args);
            let config_path = get_config_path().await;

            let operation = Operation::Increment;

            match Config::read(&config_path).await {
                Some(config) => {
                    match calculate_version(&config.current_version, change_type, operation) {
                        Ok(new_version) => {
                            println!("Current version: {}", config.current_version);
                            println!("Preview version (Increment): {}", new_version);
                        }
                        Err(e) => {
                            error!("Error calculating the version: {}", e);
                        }
                    }
                }
                None => error!("Failed to read config file at {}", config_path.display()),
            }
        }
        // -------------------------------------------------------------------------------------
        // COMANDO SHOW (sin cambios)
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
