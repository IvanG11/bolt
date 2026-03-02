
use crate::config::Config;
use anyhow::Result;
use colored::Colorize;
use std::collections::HashSet;
use std::path::Path;
use std::process::Command;

fn get_active_compose_dirs() -> HashSet<String> {
    let output = Command::new("docker")
        .args([
            "ps",
            "--format",
            "{{.Label \"com.docker.compose.project.working_dir\"}}",
        ])
        .output();

    match output {
        Ok(out) => String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter(|l| !l.is_empty())
            .map(|l| {
                Path::new(l.trim())
                    .canonicalize()
                    .unwrap_or(std::path::PathBuf::from(l.trim()))
                    .to_string_lossy()
                    .to_string()
            })
            .collect(),
        Err(_) => HashSet::new(),
    }
}

fn is_active(dir: &Path, active_dirs: &HashSet<String>) -> bool {
    let canonical = dir
        .canonicalize()
        .unwrap_or(dir.to_path_buf())
        .to_string_lossy()
        .to_string();
    active_dirs.contains(&canonical)
}

pub fn projects(config: &Config) -> Result<Vec<String>> {
    let mut names = Vec::new();
    let mut entries: Vec<_> = std::fs::read_dir(&config.projects_dir)?.flatten().collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let name = entry.file_name().to_string_lossy().to_string();
        let path = entry.path();

        if !path.is_dir() || config.ignore.contains(&name) {
            continue;
        }

        let compose = path.join("docker-compose.yml");
        if compose.exists() {
            names.push(name);
        } else {
            let has_subdirs = std::fs::read_dir(&path)
                .into_iter()
                .flatten()
                .flatten()
                .any(|s| s.path().join("docker-compose.yml").exists());
            if has_subdirs {
                names.push(name);
            }
        }
    }

    Ok(names)
}

pub fn run(config: &Config) -> Result<()> {
    let active_dirs = get_active_compose_dirs();

    let mut entries: Vec<_> = std::fs::read_dir(&config.projects_dir)?.flatten().collect();
    entries.sort_by_key(|e| e.file_name());

    // First pass: find the longest project name for column alignment
    let col_width = entries
        .iter()
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().len())
        .max()
        .unwrap_or(16)
        .max(16);

    println!(
        "{}  {}",
        "Available projects".bold(),
        config.projects_dir.display().to_string().dimmed()
    );
    println!();
    println!(
        "  {:<col_width$}  {}",
        "NAME".dimmed(),
        "STATUS   SERVICES".dimmed()
    );
    println!();

    for entry in &entries {
        let name = entry.file_name().to_string_lossy().to_string();
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let ignored = config.ignore.contains(&name);
        let compose = path.join("docker-compose.yml");

        if compose.exists() {
            let active = is_active(&path, &active_dirs);
            let status = if active { "running".green() } else { "stopped".dimmed() };
            let note = if ignored { format!("  {}", "ignored".dimmed()) } else { String::new() };
            println!("  {:<col_width$}  {}{}", name, status, note);
        } else {
            // Project with subdirectories — show subdirs inline on the same row
            let mut subdirs: Vec<_> = std::fs::read_dir(&path)
                .into_iter()
                .flatten()
                .flatten()
                .filter(|s| s.path().join("docker-compose.yml").exists())
                .collect();
            subdirs.sort_by_key(|s| s.file_name());

            if subdirs.is_empty() {
                continue;
            }

            let any_active = subdirs.iter().any(|s| is_active(&s.path(), &active_dirs));
            let status = if any_active { "running".green() } else { "stopped".dimmed() };
            let allowed = config.subdirs_for(&name);

            let subs_inline: Vec<String> = subdirs
                .iter()
                .map(|sub| {
                    let sub_name = sub.file_name().to_string_lossy().to_string();
                    let sub_active = is_active(&sub.path(), &active_dirs);
                    let skipped = allowed.map_or(false, |a| !a.contains(&sub_name));
                    let label = if skipped {
                        sub_name.dimmed().to_string()
                    } else if sub_active {
                        sub_name.green().to_string()
                    } else {
                        sub_name.normal().to_string()
                    };
                    label
                })
                .collect();

            let note = if ignored { format!("  {}", "ignored".dimmed()) } else { String::new() };

            println!(
                "  {:<col_width$}  {}{}  {}",
                name,
                status,
                note,
                subs_inline.join("  ").dimmed()
            );
        }
    }

    Ok(())
}
