use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "bolt", about = "Docker project manager", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run the setup wizard
    Setup,
    /// Stop all active projects and start the specified one (interactive picker if no project given)
    Switch {
        project: Option<String>,
        /// Keep other projects running
        #[arg(long)]
        keep: bool,
    },
    /// List available projects in projects_dir
    List {
        /// Print project names only, one per line (for shell completions)
        #[arg(long, hide = true)]
        raw: bool,
    },
    /// Show running projects and containers
    Status,
    /// Stop all active projects in projects_dir
    Stop,
    /// Restart a project (down + up) without touching others
    Restart {
        project: String,
    },
    /// Manage configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Show current configuration
    Show,
    /// Change the root projects directory
    SetDir { path: String },
    /// Add a project to the ignore list
    Ignore { project: String },
    /// Remove a project from the ignore list
    Unignore { project: String },
    /// Define which subdirectories to start for a project
    /// Example: bolt config set-subdirs acme acme,acme-api
    SetSubdirs {
        project: String,
        #[arg(value_delimiter = ',')]
        subdirs: Vec<String>,
    },
    /// Clear subdir config for a project (will start all subdirs)
    ClearSubdirs { project: String },
}
