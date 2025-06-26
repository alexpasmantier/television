use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FeatureState {
    pub enabled: bool,
    pub visible: bool,
}

impl FeatureState {
    pub const fn new(enabled: bool, visible: bool) -> Self {
        Self { enabled, visible }
    }

    pub const fn enabled() -> Self {
        Self::new(true, true)
    }

    pub const fn disabled() -> Self {
        Self::new(false, false)
    }

    pub const fn hidden() -> Self {
        Self::new(true, false)
    }

    pub fn is_active(&self) -> bool {
        self.enabled && self.visible
    }

    pub fn toggle_enabled(&mut self) {
        self.enabled = !self.enabled;
        // If disabling, also hide
        if !self.enabled {
            self.visible = false;
        }
    }

    pub fn toggle_visible(&mut self) {
        if self.enabled {
            self.visible = !self.visible;
        }
    }

    pub fn enable(&mut self) {
        self.enabled = true;
        self.visible = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
        self.visible = false;
    }

    pub fn show(&mut self) {
        if self.enabled {
            self.visible = true;
        }
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }
}

impl Default for FeatureState {
    fn default() -> Self {
        Self::disabled()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Features {
    pub preview_panel: FeatureState,
    pub help_panel: FeatureState,
    pub status_bar: FeatureState,
    pub remote_control: FeatureState,
}

impl Default for Features {
    fn default() -> Self {
        Features {
            preview_panel: FeatureState::enabled(),
            help_panel: FeatureState::hidden(),
            status_bar: FeatureState::enabled(),
            remote_control: FeatureState::hidden(),
        }
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
)]
pub enum FeatureFlags {
    PreviewPanel,
    HelpPanel,
    StatusBar,
    RemoteControl,
}

impl Features {
    /// Get the feature state for a specific feature flag
    pub fn get_state(&self, flag: FeatureFlags) -> FeatureState {
        match flag {
            FeatureFlags::PreviewPanel => self.preview_panel,
            FeatureFlags::HelpPanel => self.help_panel,
            FeatureFlags::StatusBar => self.status_bar,
            FeatureFlags::RemoteControl => self.remote_control,
        }
    }

    /// Set the feature state for a specific feature flag
    pub fn set_state(&mut self, flag: FeatureFlags, state: FeatureState) {
        match flag {
            FeatureFlags::PreviewPanel => self.preview_panel = state,
            FeatureFlags::HelpPanel => self.help_panel = state,
            FeatureFlags::StatusBar => self.status_bar = state,
            FeatureFlags::RemoteControl => self.remote_control = state,
        }
    }

    /// Check if a feature is active (enabled and visible)
    pub fn is_active(&self, flag: FeatureFlags) -> bool {
        self.get_state(flag).is_active()
    }

    /// Check if a feature is enabled (may or may not be visible)
    pub fn is_enabled(&self, flag: FeatureFlags) -> bool {
        self.get_state(flag).enabled
    }

    /// Check if a feature is visible (assumes it's enabled)
    pub fn is_visible(&self, flag: FeatureFlags) -> bool {
        self.get_state(flag).visible
    }

    /// Toggle a feature's enabled state
    pub fn toggle_enabled(&mut self, flag: FeatureFlags) {
        let mut state = self.get_state(flag);
        state.toggle_enabled();
        self.set_state(flag, state);
    }

    /// Toggle a feature's visibility
    pub fn toggle_visible(&mut self, flag: FeatureFlags) {
        let mut state = self.get_state(flag);
        state.toggle_visible();
        self.set_state(flag, state);
    }

    /// Enable a feature (makes it enabled and visible)
    pub fn enable(&mut self, flag: FeatureFlags) {
        let mut state = self.get_state(flag);
        state.enable();
        self.set_state(flag, state);
    }

    /// Disable a feature (makes it disabled and hidden)
    pub fn disable(&mut self, flag: FeatureFlags) {
        let mut state = self.get_state(flag);
        state.disable();
        self.set_state(flag, state);
    }

    /// Show a feature (makes it visible if enabled)
    pub fn show(&mut self, flag: FeatureFlags) {
        let mut state = self.get_state(flag);
        state.show();
        self.set_state(flag, state);
    }

    /// Hide a feature (makes it invisible but keeps enabled state)
    pub fn hide(&mut self, flag: FeatureFlags) {
        let mut state = self.get_state(flag);
        state.hide();
        self.set_state(flag, state);
    }
}

// Serialize/Deserialize for Features
impl Serialize for Features {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeMap;

        let mut map = serializer.serialize_map(Some(4))?;
        map.serialize_entry("preview_panel", &self.preview_panel)?;
        map.serialize_entry("help_panel", &self.help_panel)?;
        map.serialize_entry("status_bar", &self.status_bar)?;
        map.serialize_entry("remote_control", &self.remote_control)?;
        map.end()
    }
}

impl<'de> Deserialize<'de> for Features {
    fn deserialize<D>(deserializer: D) -> Result<Features, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{MapAccess, Visitor};
        use std::fmt;

        struct FeaturesVisitor;

        impl<'de> Visitor<'de> for FeaturesVisitor {
            type Value = Features;

            fn expecting(
                &self,
                formatter: &mut fmt::Formatter,
            ) -> fmt::Result {
                formatter.write_str("a map with feature states")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Features, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut features = Features::default();

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "preview_panel" => {
                            features.preview_panel = map.next_value()?;
                        }
                        "help_panel" => {
                            features.help_panel = map.next_value()?;
                        }
                        "status_bar" => {
                            features.status_bar = map.next_value()?;
                        }
                        "remote_control" => {
                            features.remote_control = map.next_value()?;
                        }
                        _ => {
                            let _: serde::de::IgnoredAny = map.next_value()?;
                        }
                    }
                }

                Ok(features)
            }
        }

        deserializer.deserialize_map(FeaturesVisitor)
    }
}

// Serialize/Deserialize for FeatureState
impl Serialize for FeatureState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeMap;

        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("enabled", &self.enabled)?;
        map.serialize_entry("visible", &self.visible)?;
        map.end()
    }
}

impl<'de> Deserialize<'de> for FeatureState {
    fn deserialize<D>(deserializer: D) -> Result<FeatureState, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{MapAccess, Visitor};
        use std::fmt;

        struct FeatureStateVisitor;

        impl<'de> Visitor<'de> for FeatureStateVisitor {
            type Value = FeatureState;

            fn expecting(
                &self,
                formatter: &mut fmt::Formatter,
            ) -> fmt::Result {
                formatter.write_str(
                    "a map with 'enabled' and 'visible' boolean fields",
                )
            }

            fn visit_map<A>(self, mut map: A) -> Result<FeatureState, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut enabled = false;
                let mut visible = false;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "enabled" => enabled = map.next_value()?,
                        "visible" => visible = map.next_value()?,
                        _ => {
                            let _: serde::de::IgnoredAny = map.next_value()?;
                        }
                    }
                }

                Ok(FeatureState::new(enabled, visible))
            }
        }

        deserializer.deserialize_map(FeatureStateVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_state_operations() {
        let mut state = FeatureState::enabled();
        assert!(state.enabled);
        assert!(state.visible);
        assert!(state.is_active());

        state.hide();
        assert!(state.enabled);
        assert!(!state.visible);
        assert!(!state.is_active());

        state.show();
        assert!(state.enabled);
        assert!(state.visible);
        assert!(state.is_active());

        state.disable();
        assert!(!state.enabled);
        assert!(!state.visible);
        assert!(!state.is_active());
    }

    #[test]
    fn test_features_operations() {
        let mut features = Features::default();

        // Test preview panel (enabled by default)
        assert!(features.is_active(FeatureFlags::PreviewPanel));

        features.hide(FeatureFlags::PreviewPanel);
        assert!(features.is_enabled(FeatureFlags::PreviewPanel));
        assert!(!features.is_visible(FeatureFlags::PreviewPanel));
        assert!(!features.is_active(FeatureFlags::PreviewPanel));

        features.show(FeatureFlags::PreviewPanel);
        assert!(features.is_active(FeatureFlags::PreviewPanel));

        // Test help panel (disabled by default)
        assert!(!features.is_active(FeatureFlags::HelpPanel));

        features.enable(FeatureFlags::HelpPanel);
        assert!(features.is_active(FeatureFlags::HelpPanel));
    }

    #[test]
    fn test_serde_serialization() {
        let features = Features::default();
        let serialized = toml::to_string(&features).unwrap();
        assert!(serialized.contains("preview_panel"));
        assert!(serialized.contains("enabled = true"));
        assert!(serialized.contains("visible = true"));
    }

    #[test]
    fn test_serde_deserialization() {
        let toml_data = r"
        [preview_panel]
        enabled = true
        visible = false
        
        [status_bar]
        enabled = true
        visible = true
        ";

        let features: Features = toml::from_str(toml_data).unwrap();
        assert!(features.is_enabled(FeatureFlags::PreviewPanel));
        assert!(!features.is_visible(FeatureFlags::PreviewPanel));
        assert!(features.is_active(FeatureFlags::StatusBar));
    }
}
