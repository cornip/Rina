use serde::{Deserialize, Serialize};
use tracing::{debug, info};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Character {
    pub name: String,
    pub preamble: String,
    pub message_examples: Vec<String>,
    pub topics: Vec<String>,
    pub style: Style,
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Style {
    pub all: Vec<String>,
    pub chat: Vec<String>,
    pub post: Vec<String>,
    pub adjectives: Vec<String>,
    pub expressions: Vec<String>,
    pub interests: Vec<String>,
    pub meme_phrases: Vec<String>,
}

impl Character {
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        info!(path = path, "Loading character configuration");
        let content = std::fs::read_to_string(path)?;
        let character: Self = toml::from_str(&content)?;
        debug!(name = character.name, "Character loaded successfully");
        Ok(character)
    }
}