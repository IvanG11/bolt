use crate::config::Config;
use anyhow::Result;
use colored::Colorize;
use std::collections::HashSet;
use std::path::Path;
use std::process::Command;

fn run_compose(compose_file: &Path, args: &[&str]) {
    Command::new("docker")
        .arg("compose")
        .arg("-f")
        .arg(compose_file)
        .args(args)
        .status()
        .ok();
}

fn is_running(compose_file: &Path) -> bool {
    Command::new("docker")
        .arg("compose")
        .arg("-f")
        .arg(compose_file)
        .args(["ps", "-q"])
        .output()
        .map(|o| !o.stdout.trim_ascii().is_empty())
        .unwrap_or(false)
}

pub fn stop_all(config: &Config) -> Result<()> {
    println!("{}", "⏹️  Stopping active projects...".yellow());

    // Query docker directly for project and working_dir of each running container
    let output = Command::new("docker")
        .args([
            "ps",
            "--format",
            "{{.Label \"com.docker.compose.project\"}}§{{.Label \"com.docker.compose.project.working_dir\"}}",
        ])
        .output()?;

    let text = String::from_utf8_lossy(&output.stdout);
    let projects_dir_canonical = config
        .projects_dir
        .canonicalize()
        .unwrap_or(config.projects_dir.clone());

    // Deduplicate by working_dir to avoid stopping the same stack twice
    let mut seen: HashSet<String> = HashSet::new();
    let mut stopped_any = false;

    for line in text.lines().filter(|l| !l.is_empty()) {
        let parts: Vec<&str> = line.splitn(2, '§').collect();
        if parts.len() < 2 {
            continue;
        }
        let project_name = parts[0].trim();
        let working_dir = parts[1].trim();

        if project_name.is_empty() || working_dir.is_empty() {
            continue;
        }

        // Skip projects in the ignore list
        if config.ignore.iter().any(|i| i == project_name) {
            continue;
        }

        // Only act on directories inside projects_dir
        let working_path = Path::new(working_dir);
        let working_canonical = working_path
            .canonicalize()
            .unwrap_or(working_path.to_path_buf());

        if !working_canonical.starts_with(&projects_dir_canonical) {
            continue;
        }

        // Deduplicate by working_dir (a stack may have multiple containers)
        if !seen.insert(working_dir.to_string()) {
            continue;
        }

        let compose = working_canonical.join("docker-compose.yml");
        if compose.exists() {
            println!("   → stopping {}", project_name.dimmed());
            run_compose(&compose, &["down"]);
            stopped_any = true;
        }
    }

    if !stopped_any {
        println!("   {} nothing running in projects_dir", "→".dimmed());
    }

    Ok(())
}

pub fn run(project: &str, keep: bool, config: &Config) -> Result<()> {
    let project_dir = config.projects_dir.join(project);

    if !project_dir.exists() {
        eprintln!("{} Project '{}' not found", "❌", project);
        std::process::exit(1);
    }

    if !keep {
        stop_all(config)?;
    }

    println!("\n{} Starting {}...", "▶️", project.bold());
    start_project(&project_dir, project, config)?;

    println!("\n{} Project '{}' is up", "✅", project.bold());
    Ok(())
}

fn stop_project(dir: &Path, project_name: &str, config: &Config) -> Result<()> {
    let compose = dir.join("docker-compose.yml");

    if compose.exists() {
        if is_running(&compose) {
            println!("   → stopping {}", project_name.dimmed());
            run_compose(&compose, &["down"]);
        }
        return Ok(());
    }

    let allowed = config.subdirs_for(project_name);

    let mut entries: Vec<_> = std::fs::read_dir(dir)?.flatten().collect();
    entries.sort_by_key(|e| e.file_name());

    for sub in entries {
        let sub_path = sub.path();
        let sub_name = sub.file_name().to_string_lossy().to_string();
        let sub_compose = sub_path.join("docker-compose.yml");

        if !sub_compose.exists() {
            continue;
        }

        if let Some(allowed_list) = allowed {
            if !allowed_list.contains(&sub_name) {
                continue;
            }
        }

        if is_running(&sub_compose) {
            println!("   → stopping {}", sub_name.dimmed());
            run_compose(&sub_compose, &["down"]);
        }
    }

    Ok(())
}

pub fn restart(project: &str, config: &Config) -> Result<()> {
    let project_dir = config.projects_dir.join(project);

    if !project_dir.exists() {
        eprintln!("{} Project '{}' not found", "❌", project);
        std::process::exit(1);
    }

    println!("{} Restarting {}...", "🔄", project.bold());
    stop_project(&project_dir, project, config)?;
    println!();
    start_project(&project_dir, project, config)?;
    println!("\n{} '{}' restarted", "✅", project.bold());

    Ok(())
}

fn start_project(dir: &Path, project_name: &str, config: &Config) -> Result<()> {
    let compose = dir.join("docker-compose.yml");

    if compose.exists() {
        println!("   → starting {}", project_name.bold());
        run_compose(&compose, &["up", "-d"]);
        return Ok(());
    }

    // Project with subdirectories
    let allowed = config.subdirs_for(project_name);

    let mut entries: Vec<_> = std::fs::read_dir(dir)?.flatten().collect();
    entries.sort_by_key(|e| e.file_name());

    let mut found_any = false;
    for sub in entries {
        let sub_path = sub.path();
        let sub_name = sub.file_name().to_string_lossy().to_string();
        let sub_compose = sub_path.join("docker-compose.yml");

        if !sub_compose.exists() {
            continue;
        }

        if let Some(allowed_list) = allowed {
            if !allowed_list.contains(&sub_name) {
                println!("   {} skipping {}", "↷".dimmed(), sub_name.dimmed());
                continue;
            }
        }

        println!("   → starting {}", sub_name.bold());
        run_compose(&sub_compose, &["up", "-d"]);
        found_any = true;
    }

    if !found_any {
        eprintln!(
            "   {} no docker-compose.yml found in '{}'",
            "⚠️", project_name
        );
    }

    Ok(())
}
