use serde::Deserialize;
use dirs;

#[derive(Deserialize, Default)]
pub struct Config {
    pub ui: UIConfig,
    pub debug: bool,
    pub history: HistoryConfig,
    pub systemd_run: SystemdRunConfig,
}

#[derive(Deserialize, Default, Clone)]
pub struct UIConfig {
    pub prompt: Option<String>,
    pub highlight_color: Option<String>,
}

#[derive(Deserialize, Default, Clone)]
pub struct HistoryConfig {
    pub enabled: bool,
    pub file: Option<String>,
}

#[derive(Deserialize, Default, Clone)]
pub struct SystemdRunConfig {
    pub parameters: Vec<String>,
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = dirs::config_dir()
            .map(|mut path| {
                path.push("fuzzyd");
                path.push("config.toml");
                path
            })
            .ok_or("Could not determine config directory")?;

        if config_path.exists() {
            let config_str = std::fs::read_to_string(config_path)?;
            Ok(toml::from_str(&config_str)?)
        } else {
            Ok(Config::default())
        }
    }

    pub fn load_from_file(path: std::path::PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let config_str = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&config_str)?)
    }
}