use serde::{Deserialize, Deserializer, Serialize, Serializer};

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub struct Features: u32 {
        const PREVIEW_PANEL      = 0b0001;
        const KEYBINDING_PANEL   = 0b0010;
        const STATUS_BAR         = 0b0100;
        const REMOTE_CONTROL     = 0b1000;
    }
}

impl Default for Features {
    fn default() -> Self {
        Features::PREVIEW_PANEL | Features::STATUS_BAR
    }
}

// Custom serialization support for human-readable feature names
impl Features {
    /// Convert a feature flag to its string name
    pub fn to_name(&self) -> Option<&'static str> {
        match *self {
            Features::PREVIEW_PANEL => Some("preview_panel"),
            Features::KEYBINDING_PANEL => Some("keybinding_panel"),
            Features::STATUS_BAR => Some("status_bar"),
            Features::REMOTE_CONTROL => Some("remote_control"),
            _ => None,
        }
    }

    /// Parse a feature name string to its flag
    pub fn parse_name(name: &str) -> Option<Features> {
        match name {
            "preview_panel" => Some(Features::PREVIEW_PANEL),
            "keybinding_panel" => Some(Features::KEYBINDING_PANEL),
            "status_bar" => Some(Features::STATUS_BAR),
            "remote_control" => Some(Features::REMOTE_CONTROL),
            _ => None,
        }
    }

    /// Get all individual features as a vector of names
    pub fn to_names(&self) -> Vec<&'static str> {
        let mut names = Vec::new();
        for feature in [
            Features::PREVIEW_PANEL,
            Features::KEYBINDING_PANEL,
            Features::STATUS_BAR,
            Features::REMOTE_CONTROL,
        ] {
            if self.contains(feature) {
                if let Some(name) = feature.to_name() {
                    names.push(name);
                }
            }
        }
        names
    }

    /// Create Features from a vector of feature names
    pub fn from_names<I>(names: I) -> Result<Features, String>
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        let mut features = Features::empty();
        for name in names {
            match Features::parse_name(name.as_ref()) {
                Some(feature) => features.insert(feature),
                None => {
                    return Err(format!("Unknown feature: {}", name.as_ref()));
                }
            }
        }
        Ok(features)
    }
}

// Custom serde implementation to support array of strings
impl Serialize for Features {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize as array of feature names for readability
        self.to_names().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Features {
    fn deserialize<D>(deserializer: D) -> Result<Features, D::Error>
    where
        D: Deserializer<'de>,
    {
        let names: Vec<String> = Vec::deserialize(deserializer)?;
        Features::from_names(names).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_names() {
        assert_eq!(Features::PREVIEW_PANEL.to_name(), Some("preview_panel"));
        assert_eq!(
            Features::KEYBINDING_PANEL.to_name(),
            Some("keybinding_panel")
        );
        assert_eq!(Features::STATUS_BAR.to_name(), Some("status_bar"));
        assert_eq!(Features::REMOTE_CONTROL.to_name(), Some("remote_control"));
    }

    #[test]
    fn test_parse_names() {
        assert_eq!(
            Features::parse_name("preview_panel"),
            Some(Features::PREVIEW_PANEL)
        );
        assert_eq!(
            Features::parse_name("status_bar"),
            Some(Features::STATUS_BAR)
        );
        assert_eq!(Features::parse_name("invalid"), None);
    }

    #[test]
    fn test_from_names() {
        let features =
            Features::from_names(["preview_panel", "status_bar"]).unwrap();
        assert!(features.contains(Features::PREVIEW_PANEL));
        assert!(features.contains(Features::STATUS_BAR));
        assert!(!features.contains(Features::KEYBINDING_PANEL));
        assert!(!features.contains(Features::REMOTE_CONTROL));
    }

    #[test]
    fn test_to_names() {
        let features = Features::PREVIEW_PANEL | Features::STATUS_BAR;
        let names = features.to_names();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"preview_panel"));
        assert!(names.contains(&"status_bar"));
    }

    #[test]
    fn test_serde_serialization() {
        #[derive(serde::Serialize)]
        struct TestConfig {
            features: Features,
        }

        let config = TestConfig {
            features: Features::PREVIEW_PANEL | Features::STATUS_BAR,
        };
        let serialized = toml::to_string(&config).unwrap();
        assert!(serialized.contains("preview_panel"));
        assert!(serialized.contains("status_bar"));
    }

    #[test]
    fn test_serde_deserialization_names() {
        #[derive(serde::Deserialize)]
        struct TestConfig {
            features: Features,
        }

        let toml_data = r#"features = ["preview_panel", "status_bar"]"#;
        let config: TestConfig = toml::from_str(toml_data).unwrap();
        assert!(config.features.contains(Features::PREVIEW_PANEL));
        assert!(config.features.contains(Features::STATUS_BAR));
        assert!(!config.features.contains(Features::KEYBINDING_PANEL));
    }
}
