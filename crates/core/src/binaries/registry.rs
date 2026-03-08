use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Strategy for obtaining a binary on a platform.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "strategy")]
pub enum BinarySource {
    /// Direct download from URL (Windows).
    #[serde(rename = "download")]
    Download {
        url: String,
        #[serde(rename = "type")]
        archive_type: String,
    },
    /// Use system-installed binary (macOS).
    #[serde(rename = "system")]
    System {
        install_hint: String,
    },
}

/// Registry of known binary sources, keyed by service → version → platform.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryRegistry {
    #[serde(flatten)]
    pub services: HashMap<String, HashMap<String, HashMap<String, BinarySource>>>,
}

impl BinaryRegistry {
    /// Load the built-in registry.
    pub fn load_builtin() -> Self {
        let json = include_str!("registry.json");
        serde_json::from_str(json).expect("Built-in registry.json is invalid")
    }

    /// Look up a binary source for the current platform.
    pub fn lookup(&self, service: &str, version: &str) -> Option<&BinarySource> {
        let platform_key = format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH);
        self.services
            .get(service)?
            .get(version)?
            .get(&platform_key)
    }
}
