use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct AppConfig {
    pub api_key: Option<String>,
    pub language: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            language: "zh_CN".to_string(),
        }
    }
}

pub fn load_config() -> AppConfig {
    confy::load("OCR-eg", None).unwrap_or_default()
}

pub fn save_config(config: &AppConfig) -> anyhow::Result<()> {
    confy::store("OCR-eg", None, config)?;
    Ok(())
}
