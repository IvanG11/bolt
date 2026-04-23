use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ProjectConfig {
    pub subdirs: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Config {
    #[serde(default)]
    pub projects_dir: PathBuf,
    #[serde(default)]
    pub ignore: Vec<String>,
    #[serde(default)]
    pub projects: HashMap<String, ProjectConfig>,
}

impl Config {
    pub fn is_configured(&self) -> bool {
        !self.projects_dir.as_os_str().is_empty()
    }

    pub fn path() -> PathBuf {
        dirs::config_dir()
            .unwrap()
            .join("bolt")
            .join("config.toml")
    }

    pub fn load() -> Result<Self> {
        let path = Self::path();
        if !path.exists() {
            return Ok(Config::default());
        }
        let content = std::fs::read_to_string(&path)?;
        Ok(toml::from_str(&content)?)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::path();
        std::fs::create_dir_all(path.parent().unwrap())?;
        std::fs::write(&path, toml::to_string_pretty(self)?)?;
        Ok(())
    }

    /// Allowed subdirs for a project, or None if unrestricted
    pub fn subdirs_for(&self, project: &str) -> Option<&Vec<String>> {
        self.projects
            .get(project)
            .map(|p| &p.subdirs)
            .filter(|s| !s.is_empty())
    }
}
