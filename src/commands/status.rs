use anyhow::Result;
use colored::Colorize;
use std::collections::{BTreeMap, HashSet};
use std::process::Command;

struct Container {
    name: String,
    image: String,
    status: String,
    ports: String,
}

/// Extract only host-mapped ports, deduplicate IPv4/IPv6, simplify same-port mappings.
/// "8443/tcp, 0.0.0.0:8000->8080/tcp, [::]:8000->8080/tcp" → "8000→8080"
fn parse_ports(raw: &str) -> String {
    let mut seen = HashSet::new();
    let mut result = Vec::new();

    for part in raw.split(", ") {
        let part = part.trim();
        let Some(arrow) = part.find("->") else { continue };

        let host_port = part[..arrow].rsplit(':').next().unwrap_or(&part[..arrow]);
        let container = part[arrow + 2..]
            .trim_end_matches("/tcp")
            .trim_end_matches("/udp");

        if seen.insert(format!("{}-{}", host_port, container)) {
            if host_port == container {
                result.push(host_port.to_string());
            } else {
                result.push(format!("{}→{}", host_port, container));
            }
        }
    }

    result.join("  ")
}

/// "Up 2 minutes (healthy)" → "Up 2m (healthy)"
fn shorten_status(s: &str) -> String {
    s.replace(" minutes", "m")
        .replace(" minute", "m")
        .replace(" hours", "h")
        .replace(" hour", "h")
        .replace(" seconds", "s")
        .replace(" second", "s")
        .replace(" days", "d")
        .replace(" day", "d")
        .replace(" weeks", "w")
        .replace(" week", "w")
}

pub fn run() -> Result<()> {
    println!("{}", "Active containers".bold());
    println!();

    let output = Command::new("docker")
        .args([
            "ps",
            "--format",
            "{{.Label \"com.docker.compose.project\"}}§{{.Names}}§{{.Image}}§{{.Status}}§{{.Ports}}",
        ])
        .output()?;

    let text = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = text.lines().filter(|l| !l.is_empty()).collect();

    if lines.is_empty() {
        println!("  {} no containers running", "→".dimmed());
        return Ok(());
    }

    let mut groups: BTreeMap<String, Vec<Container>> = BTreeMap::new();

    for line in &lines {
        let parts: Vec<&str> = line.splitn(5, '§').collect();
        if parts.len() < 5 {
            continue;
        }

        let project = parts[0].trim();
        if project.is_empty() {
            continue;
        }

        groups.entry(project.to_string()).or_default().push(Container {
            name: parts[1].trim().to_string(),
            image: parts[2].trim().to_string(),
            status: shorten_status(parts[3].trim()),
            ports: parse_ports(parts[4].trim()),
        });
    }

    // Compute column widths across all containers
    let all: Vec<&Container> = groups.values().flatten().collect();
    let name_w = all.iter().map(|c| c.name.len()).max().unwrap_or(4).max(4);
    let image_w = all.iter().map(|c| c.image.len()).max().unwrap_or(5).max(5);
    let status_w = all.iter().map(|c| c.status.len()).max().unwrap_or(6).max(6);

    for (project, containers) in &groups {
        println!("  {}", project.bold());
        for c in containers {
            let status_colored = if c.status.starts_with("Up") {
                c.status.green().to_string()
            } else {
                c.status.yellow().to_string()
            };
            // Pad status manually — ANSI codes break format width
            let pad = " ".repeat(status_w.saturating_sub(c.status.len()));
            println!(
                "    {:<name_w$}  {:<image_w$}  {}{}  {}",
                c.name,
                c.image,
                status_colored,
                pad,
                c.ports.dimmed()
            );
        }
        println!();
    }

    Ok(())
}
