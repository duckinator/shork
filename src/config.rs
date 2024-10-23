use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Config {
    pub server: String,
    pub token: String,
    pub folder: String,
}

impl Config {
    pub fn auth(&self) -> String {
        format!("MediaBrowser Token=\"{}\"", self.token)
    }
}
