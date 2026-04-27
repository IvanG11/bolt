mod cli;
mod commands;
mod config;

use clap::Parser;
use cli::{Cli, Commands, ConfigAction};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let mut config = config::Config::load()?;

    let needs_config = matches!(
        cli.command,
        Commands::Switch { .. } | Commands::Start { .. } | Commands::List { .. }
        | Commands::Stop | Commands::Ui { .. } | Commands::Build { .. }
    );
    if needs_config && !config.is_configured() {
        commands::onboarding::run(&mut config)?;
    }

    match cli.command {
        Commands::Setup => {
            commands::onboarding::run(&mut config)?;
        }
        Commands::Switch { project } => {
            let project = match project {
                Some(p) => p,
                None => {
                    let projects = commands::list::projects(&config)?;
                    if projects.is_empty() {
                        eprintln!("No projects found in {}", config.projects_dir.display());
                        std::process::exit(1);
                    }
                    let idx = dialoguer::FuzzySelect::new()
                        .with_prompt("Switch to")
                        .items(&projects)
                        .interact()?;
                    projects[idx].clone()
                }
            };
            commands::switch::run(&project, false, &config)?;
        }
        Commands::Start { project } => {
            let project = match project {
                Some(p) => p,
                None => {
                    let projects = commands::list::projects(&config)?;
                    if projects.is_empty() {
                        eprintln!("No projects found in {}", config.projects_dir.display());
                        std::process::exit(1);
                    }
                    let idx = dialoguer::FuzzySelect::new()
                        .with_prompt("Start")
                        .items(&projects)
                        .interact()?;
                    projects[idx].clone()
                }
            };
            commands::switch::run(&project, true, &config)?;
        }
        Commands::List { raw } => {
            if raw {
                for name in commands::list::projects(&config)? {
                    println!("{}", name);
                }
            } else {
                commands::list::run(&config)?;
            }
        }
        Commands::Stop => {
            commands::switch::stop_all(&config)?;
        }
        Commands::Restart { project } => {
            commands::switch::restart(&project, &config)?;
        }
        Commands::Build { project } => {
            commands::switch::build(&project, &config)?;
        }
        Commands::Ui { port, daemon, stop } => {
            commands::ui::run(config, port, daemon, stop)?;
        }
        Commands::Status => {
            commands::status::run()?;
        }
        Commands::Config { action } => match action {
            ConfigAction::Show => {
                println!("{:#?}", config);
                println!(
                    "\n  (file: {})",
                    config::Config::path().display()
                );
            }
            ConfigAction::SetDir { path } => {
                config.projects_dir = path.into();
                config.save()?;
                println!("✅ projects_dir updated");
            }
            ConfigAction::Ignore { project } => {
                if !config.ignore.contains(&project) {
                    config.ignore.push(project.clone());
                    config.save()?;
                    println!("✅ '{}' added to ignore list", project);
                } else {
                    println!("'{}' is already in the ignore list", project);
                }
            }
            ConfigAction::Unignore { project } => {
                config.ignore.retain(|p| p != &project);
                config.save()?;
                println!("✅ '{}' removed from ignore list", project);
            }
            ConfigAction::SetSubdirs { project, subdirs } => {
                let entry = config.projects.entry(project.clone()).or_default();
                entry.subdirs = subdirs.clone();
                config.save()?;
                println!(
                    "✅ subdirs for '{}': {}",
                    project,
                    subdirs.join(", ")
                );
            }
            ConfigAction::ClearSubdirs { project } => {
                if let Some(p) = config.projects.get_mut(&project) {
                    p.subdirs.clear();
                    config.save()?;
                    println!("✅ subdirs for '{}' cleared (will start all)", project);
                } else {
                    println!("'{}' has no subdir config", project);
                }
            }
        },
    }

    Ok(())
}
