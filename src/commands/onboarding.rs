use crate::config::Config;
use anyhow::Result;
use colored::Colorize;
use rustyline::completion::FilenameCompleter;
use rustyline::{CompletionType, Editor};
use rustyline::{Completer, Helper, Highlighter, Hinter, Validator};
use std::path::PathBuf;

#[derive(Helper, Completer, Hinter, Validator, Highlighter)]
struct PathHelper(#[rustyline(Completer)] FilenameCompleter);

fn read_line(prompt: &str, initial: &str) -> Result<String> {
    let mut rl: Editor<(), _> = Editor::new()?;
    let input = rl.readline_with_initial(prompt, (initial, ""))?;
    Ok(input.trim().to_string())
}

fn read_path(prompt: &str, initial: &str) -> Result<String> {
    let config = rustyline::Config::builder()
        .completion_type(CompletionType::List)
        .build();
    let mut rl = Editor::with_config(config)?;
    rl.set_helper(Some(PathHelper(FilenameCompleter::new())));
    let input = rl.readline_with_initial(prompt, (initial, ""))?;
    Ok(input.trim().to_string())
}

fn expand_tilde(path: &str) -> PathBuf {
    if path.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(&path[2..]);
        }
    } else if path == "~" {
        if let Some(home) = dirs::home_dir() {
            return home;
        }
    }
    PathBuf::from(path)
}

enum Shell { Bash, Zsh, Fish }

fn detect_shell() -> Option<Shell> {
    let shell_bin = std::env::var("SHELL").unwrap_or_default();
    if shell_bin.contains("fish") {
        Some(Shell::Fish)
    } else if shell_bin.contains("zsh") {
        Some(Shell::Zsh)
    } else if shell_bin.contains("bash") {
        Some(Shell::Bash)
    } else {
        None
    }
}

fn install_completions(shell: Shell) -> Result<()> {
    let script = match shell {
        Shell::Fish => {
            "# bolt project completions\n\
             complete -c bolt -f\n\
             function __bolt_projects\n\
             \x20   bolt list --raw 2>/dev/null\n\
             end\n\
             complete -c bolt -n '__fish_seen_subcommand_from switch restart' -f -a '(__bolt_projects)'\n"
        }
        Shell::Zsh => {
            "#compdef bolt\n\
             # bolt project completions\n\
             _bolt() {\n\
             \x20 case $words[2] in\n\
             \x20   switch|restart)\n\
             \x20     local -a projects\n\
             \x20     projects=(${(f)\"$(bolt list --raw 2>/dev/null)\"})\n\
             \x20     _describe 'project' projects\n\
             \x20     ;;\n\
             \x20 esac\n\
             }\n\
             _bolt\n"
        }
        Shell::Bash => {
            "# bolt project completions\n\
             _bolt_completions() {\n\
             \x20 local sub=${COMP_WORDS[1]}\n\
             \x20 if [[ \"$sub\" == \"switch\" || \"$sub\" == \"restart\" ]]; then\n\
             \x20   COMPREPLY=($(compgen -W \"$(bolt list --raw 2>/dev/null)\" -- \"${COMP_WORDS[COMP_CWORD]}\"))\n\
             \x20 fi\n\
             }\n\
             complete -F _bolt_completions bolt\n"
        }
    };

    let home = dirs::home_dir().expect("could not find home directory");
    let dest = match shell {
        Shell::Fish => home.join(".config/fish/completions/bolt.fish"),
        Shell::Zsh => home.join(".zfunc/_bolt"),
        Shell::Bash => home.join(".local/share/bash-completion/completions/bolt"),
    };
    std::fs::create_dir_all(dest.parent().unwrap())?;
    std::fs::write(&dest, script)?;
    println!("✅ Completions installed to {}", dest.display());

    Ok(())
}

pub fn run(config: &mut Config) -> Result<()> {
    println!("{}", "Welcome to bolt!".bold());
    if config.is_configured() {
        println!("Existing configuration found — edit or press Enter to keep current values.\n");
    } else {
        println!("Let's set up your configuration.\n");
    }

    let dir_initial = if config.is_configured() {
        config.projects_dir.to_string_lossy().into_owned()
    } else {
        String::new()
    };
    let ignore_initial = config.ignore.join(", ");

    // Ask for projects directory
    let projects_dir = loop {
        let input = read_path("Projects directory: ", &dir_initial)?;
        if input.is_empty() {
            println!("{}", "  Please enter a directory.".yellow());
            continue;
        }
        let path = expand_tilde(&input);
        if !path.exists() {
            println!("  {} Directory '{}' does not exist.", "⚠️", path.display());
            let confirm = read_line("  Create it? [y/N]: ", "")?;
            if confirm.to_lowercase() == "y" {
                std::fs::create_dir_all(&path)?;
                break path;
            }
        } else {
            break path;
        }
    };

    config.projects_dir = projects_dir;

    // Ask for ignore list
    let ignore_input = read_line(
        "Directories to ignore (comma-separated, leave empty for none): ",
        &ignore_initial,
    )?;
    config.ignore = ignore_input
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    config.save()?;
    println!(
        "\n{} Configuration saved to {}",
        "✅",
        Config::path().display()
    );

    // Offer to install shell completions
    println!();
    let shell_label = match detect_shell() {
        Some(Shell::Fish) => Some("fish"),
        Some(Shell::Zsh) => Some("zsh"),
        Some(Shell::Bash) => Some("bash"),
        None => None,
    };
    let prompt = match shell_label {
        Some(name) => format!("Install shell completions for {}? [Y/n]: ", name),
        None => "Install shell completions? [Y/n]: ".to_string(),
    };
    let answer = read_line(&prompt, "")?;
    if answer.is_empty() || answer.to_lowercase() == "y" {
        match detect_shell() {
            Some(shell) => install_completions(shell)?,
            None => println!("{} Could not detect shell. Skipping.", "⚠️"),
        }
    }
    println!();

    Ok(())
}
