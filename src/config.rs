use std::error::Error;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    pub server: String,
    pub token: String,
    pub folder: String,
}

impl Config {
    pub fn load(path: &str) -> Result<Self, Box<dyn Error>> {
        let config = std::fs::read_to_string(path)?;
        Ok(Self::from_str(&config)?)
    }

    pub fn from_str(config: &str) -> Result<Self, Box<dyn Error>> {
        Ok(toml::from_str(config)?)
    }

    /*pub fn to_str(&self) -> Result<String, Box<dyn Error>> {
        Ok(toml::to_string(&self)?)
    }*/
}
