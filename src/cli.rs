use clap::{Parser, Args, Subcommand, ArgAction};

const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");

#[derive(Parser)]
#[command(author = AUTHORS, version = VERSION, about, long_about = DESCRIPTION)] 
pub struct Cli {
    #[arg(short, long, action = ArgAction::SetTrue)]
    /// Enables debug messages for the 'vampus' application.
    pub debug: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Increments the project version (updates the version number in the configuration).
    // Usamos la nueva estructura compartida
    Upgrade(VersionArgs),
    
    /// Shows the resulting project version without applying the change.
    Preview(VersionArgs),

    /// Sets the project version back to a calculated previous version and updates the files.
    // Usamos la nueva estructura compartida
    Downgrade(VersionArgs), 
    
    /// Displays the current version of the project.
    Show,
}

#[derive(Args)]
/// Arguments specific to version manipulation commands (upgrade, downgrade, preview).
// Creamos una única estructura para ambos comandos
pub struct VersionArgs {
    // --- Opciones de Versión Mutuamente Excluyentes ---
    
    /// Increments/Decrements the PATCH version (bug fixes).
    #[arg(long, action = ArgAction::SetTrue, group = "VERSION_TYPE")]
    pub patch: bool,
    
    /// Increments/Decrements the MINOR version (new features).
    #[arg(long, action = ArgAction::SetTrue, group = "VERSION_TYPE")]
    pub minor: bool,
    
    /// Increments/Decrements the MAJOR version (breaking changes).
    #[arg(long, action = ArgAction::SetTrue, group = "VERSION_TYPE")]
    pub major: bool,
}
