use std::fs;
use std::path::PathBuf;

use anyhow::Context;

use crate::config::AgentConfig;

const APP_DIR_NAME: &str = "Now Playing";
const CONFIG_FILE_NAME: &str = "config.toml";

pub struct ConfigStore {
    pub config_path: PathBuf,
    pub app_dir: PathBuf,
}

impl ConfigStore {
    pub fn load_or_create() -> anyhow::Result<(Self, AgentConfig)> {
        let app_dir = app_support_dir()?.join(APP_DIR_NAME);
        fs::create_dir_all(&app_dir).context("failed to create application support directory")?;

        let config_path = app_dir.join(CONFIG_FILE_NAME);
        let store = Self {
            config_path: config_path.clone(),
            app_dir,
        };

        if !config_path.exists() {
            let template = AgentConfig::default_template();
            store.save(&template)?;
            return Ok((store, template));
        }

        let config = store.load()?;
        Ok((store, config))
    }

    pub fn load(&self) -> anyhow::Result<AgentConfig> {
        let contents =
            fs::read_to_string(&self.config_path).context("failed to read config file")?;
        toml::from_str(&contents).context("failed to parse config file")
    }

    pub fn save(&self, config: &AgentConfig) -> anyhow::Result<()> {
        let contents = toml::to_string_pretty(config).context("failed to serialize config")?;
        fs::write(&self.config_path, contents).context("failed to write config file")
    }

    pub fn log_dir(&self) -> PathBuf {
        self.app_dir.join("logs")
    }
}

fn app_support_dir() -> anyhow::Result<PathBuf> {
    directories::ProjectDirs::from("", "", APP_DIR_NAME)
        .map(|dirs| dirs.data_dir().to_path_buf())
        .context("failed to resolve application support directory")
}
