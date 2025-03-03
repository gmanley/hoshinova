use crate::module::TaskStatus;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, TS, Serialize, Deserialize, Debug)]
#[ts(export, export_to = "web/src/bindings/")]
pub struct Config {
    pub ytarchive: YtarchiveConfig,
    pub scraper: ScraperConfig,
    pub notifier: Option<NotifierConfig>,
    pub webserver: Option<WebserverConfig>,
    pub channel: Vec<ChannelConfig>,

    #[serde(skip)]
    #[ts(skip)]
    config_path: String,
}

#[derive(Clone, TS, Serialize, Deserialize, Debug)]
#[ts(export, export_to = "web/src/bindings/")]
pub struct YtarchiveConfig {
    pub executable_path: String,
    pub working_directory: String,
    pub args: Vec<String>,
    pub quality: String,
    #[serde(with = "humantime_serde")]
    #[serde(default = "default_delay_start")]
    #[ts(type = "string")]
    pub delay_start: std::time::Duration,
}

fn default_delay_start() -> std::time::Duration {
    std::time::Duration::from_secs(1)
}

#[derive(Clone, TS, Serialize, Deserialize, Debug)]
#[ts(export, export_to = "web/src/bindings/")]
pub struct ScraperConfig {
    pub rss: ScraperRSSConfig,
}

#[derive(Clone, TS, Serialize, Deserialize, Debug)]
#[ts(export, export_to = "web/src/bindings/")]
pub struct ScraperRSSConfig {
    #[serde(with = "humantime_serde")]
    #[ts(type = "string")]
    pub poll_interval: std::time::Duration,
    #[serde(with = "humantime_serde")]
    #[serde(default = "default_ignore_older_than")]
    #[ts(type = "string")]
    pub ignore_older_than: std::time::Duration,
}

fn default_ignore_older_than() -> std::time::Duration {
    std::time::Duration::from_secs(60 * 60 * 24)
}

#[derive(Clone, TS, Serialize, Deserialize, Debug)]
#[ts(export, export_to = "web/src/bindings/")]
pub struct NotifierConfig {
    pub discord: Option<NotifierDiscordConfig>,
}

#[derive(Clone, TS, Serialize, Deserialize, Debug)]
#[ts(export, export_to = "web/src/bindings/")]
pub struct NotifierDiscordConfig {
    pub webhook_url: String,
    pub notify_on: Vec<TaskStatus>,
}

#[derive(Clone, TS, Serialize, Deserialize, Debug)]
#[ts(export, export_to = "web/src/bindings/")]
pub struct WebserverConfig {
    pub bind_address: String,
}

#[derive(Clone, TS, Serialize, Deserialize, Debug)]
#[ts(export, export_to = "web/src/bindings/")]
pub struct ChannelConfig {
    pub id: String,
    pub name: String,
    #[serde(with = "serde_regex")]
    #[ts(type = "string[]")]
    pub filters: Vec<regex::Regex>,
    #[serde(default = "default_false")]
    pub match_description: bool,
    pub outpath: String,
    /// If not present, will be fetched during runtime.
    pub picture_url: Option<String>,
}

fn default_false() -> bool {
    false
}

pub async fn load_config(path: &str) -> Result<Config> {
    let config = tokio::fs::read_to_string(path).await?;
    let mut config: Config = toml::from_str(&config)?;
    config.config_path = path.to_string();
    Ok(config)
}

impl Config {
    /// Reads the config file and replaces the current config with the new one.
    pub async fn reload(&mut self) -> Result<()> {
        info!("Reloading config");
        let config = load_config(&self.config_path)
            .await
            .context("Failed to load config")?;
        *self = config;
        Ok(())
    }

    /// Reads and returns the source TOML file from the config path. There are
    /// no guarantees that the returned TOML corresponds to the current config,
    /// as it might have been changed since the last time it was read.
    pub async fn get_source_toml(&self) -> Result<String> {
        tokio::fs::read_to_string(&self.config_path)
            .await
            .map_err(|e| e.into())
    }

    /// Writes the provided TOML string to the config path, and reloads the
    /// config.
    pub async fn set_source_toml(&mut self, source_toml: &str) -> Result<()> {
        // Try to deserialize the provided TOML string. If it fails, we don't
        // want to write it to the config file.
        let _: Config =
            toml::from_str(source_toml).context("Failed to deserialize provided TOML")?;

        // Write the provided TOML string to the config file.
        tokio::fs::write(&self.config_path, source_toml)
            .await
            .context("Failed to write config file")?;

        // Reload the config.
        self.reload().await
    }
}
