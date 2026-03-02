# bolt — Docker Project Manager

CLI to manage Docker Compose projects.

## Installation

```bash
curl -fsSL https://raw.githubusercontent.com/IvanG11/bolt/main/install.sh | sudo bash
```

### From source (requires Rust)

```bash
git clone https://github.com/IvanG11/bolt
cd bolt
make build
sudo make install
```

## Commands

| Command                                     | Description                                           |
| ------------------------------------------- | ----------------------------------------------------- |
| `bolt setup`                                | Run the setup wizard                                  |
| `bolt switch <project>`                     | Stop all active projects and start the specified one  |
| `bolt switch <project> --keep`              | Start without stopping others                         |
| `bolt list`                                 | List projects with status ▶/⏹                         |
| `bolt status`                               | Show active containers grouped by project             |
| `bolt stop`                                 | Stop all active projects in projects_dir              |
| `bolt restart <project>`                    | Restart a project (down + up) without touching others |
| `bolt config show`                          | Show current configuration                            |
| `bolt config set-dir <path>`                | Change the root projects directory                    |
| `bolt config ignore <project>`              | Add a project to the ignore list                      |
| `bolt config unignore <project>`            | Remove a project from the ignore list                 |
| `bolt config set-subdirs <project> <s1,s2>` | Define which subdirs to start for a project           |
| `bolt config clear-subdirs <project>`       | Reset to starting all subdirs                         |

## Shell completions

`bolt switch <tab>` autocompletes project names. Shell completions are installed automatically during `bolt setup`.

## Configuration

The config file is created automatically at:
`~/.config/bolt/config.toml`

```toml
projects_dir = "/home/user/Projects"
ignore = ["docker-services"]

[projects.acme]
subdirs = ["acme", "acme-api"]
```

## Stop behaviour

- Only stops projects inside `projects_dir`
- Queries `docker ps` directly — no directory iteration
- Respects the `ignore` list — never touches those projects
- Never touches containers outside `projects_dir`

## Projects with subdirectories

If a project has no `docker-compose.yml` at the root but in subdirectories:

```
Projects/
└── acme/
    ├── acme/          ← docker-compose.yml
    ├── acme-api/      ← docker-compose.yml
    ├── acme-legacy/   ← docker-compose.yml (will be skipped)
    └── acme-old/      ← docker-compose.yml (will be skipped)
```

Configure which subdirs to start:

```bash
bolt config set-subdirs acme acme,acme-api
```

## Publishing a new version

**1. Update the version in `Cargo.toml`:**

```toml
version = "1.1.0"
```

**2. Commit the change:**

```bash
git add Cargo.toml
git commit -m "bump v1.1.0"
```

**3. Publish:**

```bash
make release
```

This creates the tag `v1.1.0` and pushes it. GitHub Actions detects the tag, builds the binary and uploads it to GitHub Releases automatically.

## License

bolt is open-sourced software licensed under the [MIT license](https://opensource.org/licenses/MIT).
