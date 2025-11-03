use clap::{Parser, Args, Subcommand, ArgAction};

const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");

#[derive(Parser)]
#[command(author = AUTHORS, version = VERSION, about, long_about = None)]
pub struct Cli {
    #[arg(short, long, action = ArgAction::SetTrue)]
    /// debug podcli
    pub debug: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Upgrade(UpgradeArgs),
    Preview(UpgradeArgs),
    Show,
}

#[derive(Args)]
struct UpgradeArgs {
    /// Set the change type (major, minor, patch)
    #[arg(short, long)]
    change: Option<String>,
}


