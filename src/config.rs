use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    // Deprecated but kept for migration
    pub api_key: Option<String>,
    
    pub language: String,
    
    // New fields
    pub active_provider: String,
    pub api_keys: HashMap<String, String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            language: "zh_CN".to_string(),
            active_provider: "mistral".to_string(),
            api_keys: HashMap::new(),
        }
    }
}

pub fn load_config() -> AppConfig {
    let mut config: AppConfig = confy::load("ocr-eg", None).unwrap_or_default();
    
    // Migration: If old api_key exists but not in map, move it
    if let Some(old_key) = &config.api_key {
        if !config.api_keys.contains_key("mistral") {
            config.api_keys.insert("mistral".to_string(), old_key.clone());
        }
    }
    
    // Ensure default active provider
    if config.active_provider.is_empty() {
        config.active_provider = "mistral".to_string();
    }
    
    config
}

pub fn save_config(config: &AppConfig) -> anyhow::Result<()> {
    confy::store("ocr-eg", None, config)?;
    Ok(())
}