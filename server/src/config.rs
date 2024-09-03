use dirs::config_dir;
pub use dry_console_dto::config::*;
use ron::de::from_str;
use std::error::Error;
use std::fs;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use tracing::{debug, info};

pub fn default_config_path() -> String {
    let mut path = config_dir().expect("Could not find platform specific config directory.");
    path.push("dry_console/config.ron");
    path.to_string_lossy().into_owned()
}

pub fn load_config(config_path: &str) -> Result<Config, Box<dyn Error>> {
    let path = Path::new(config_path);
    let mut config;
    if path.exists() {
        // If the file exists, attempt to read and parse it.
        let config_contents = fs::read_to_string(config_path)?;
        config = from_str(&config_contents)?;
    } else {
        // If the file does not exist, create a default config and save it.
        config = Config::default();
        // Add the d.rymcg.tech section:
        config.sections.insert(
            ConfigSection::DRymcgTech,
            ConfigData::DRymcgTech(DRymcgTechConfig {
                root_dir: None,
                ..Default::default()
            }),
        );
        // Save the config:
        save_config(&config, config_path)?;
    }
    debug!("Loaded config: {}", config_path);
    Ok(config)
}

pub fn save_config(config: &Config, config_path: &str) -> Result<(), Box<dyn Error>> {
    let default_path = default_config_path();
    if config_path == default_path {
        if let Some(parent) = Path::new(config_path).parent() {
            fs::create_dir_all(parent)?;
        }
    }

    let ron_string = ron::ser::to_string_pretty(config, ron::ser::PrettyConfig::default())?;
    // Check if the file already exists and read its contents
    let mut existing_content = String::new();
    let file_path = Path::new(config_path);
    if file_path.exists() {
        let mut file = fs::File::open(config_path)?;
        file.read_to_string(&mut existing_content)?;
    }

    // Only write the file if the content has changed
    if existing_content != ron_string {
        let mut file = fs::File::create(config_path)?;
        file.write_all(ron_string.as_bytes())?;
        info!("Saved config file: {}", config_path);
    }
    Ok(())
}
