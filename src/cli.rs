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
    Upgrade(UpgradeArgs),
    
    /// Shows the resulting project version without applying the change.
    Preview(UpgradeArgs),
    
    /// Displays the current version of the project.
    Show,
}

#[derive(Args)]
/// Arguments specific to the 'upgrade' and 'preview' commands.
// Para forzar que el usuario elija al menos una opción, añade:
// #[clap(group = clap::ArgGroup::new("VERSION_TYPE").required(true))]
// Si no quieres forzar que elija una, el valor por defecto se gestiona en la lógica.
pub struct UpgradeArgs {
    // --- Opciones de Versión Mutuamente Excluyentes ---
    
    /// Increments the PATCH version (bug fixes).
    #[arg(long, action = ArgAction::SetTrue, group = "VERSION_TYPE")]
    pub patch: bool,
    
    /// Increments the MINOR version (new features).
    #[arg(long, action = ArgAction::SetTrue, group = "VERSION_TYPE")]
    pub minor: bool,
    
    /// Increments the MAJOR version (breaking changes).
    #[arg(long, action = ArgAction::SetTrue, group = "VERSION_TYPE")]
    pub major: bool,
}
