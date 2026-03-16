use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Bot configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Database path
    pub database_path: String,
    
    /// Log level
    pub log_level: String,
    
    /// Auto-reply enabled
    pub auto_reply: bool,
    
    /// Auto-reply message
    pub auto_reply_message: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database_path: "ruwa-bot.db".to_string(),
            log_level: "info".to_string(),
            auto_reply: false,
            auto_reply_message: "Thank you for your message! I'm a bot and will respond soon.".to_string(),
        }
    }
}

impl Config {
    /// Load configuration from file or use defaults
    pub fn load() -> Self {
        let config_path = PathBuf::from("config.toml");
        
        if config_path.exists() {
            // Try to load from file
            if let Ok(content) = std::fs::read_to_string(&config_path) {
                if let Ok(config) = toml::from_str(&content) {
                    log::info!("📄 Configuration loaded from config.toml");
                    return config;
                }
            }
        }
        
        // Use defaults
        log::info!("📄 Using default configuration");
        Config::default()
    }
    
    /// Save configuration to file
    pub fn save(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}
