//! # Television UI Features System
//!
//! **Table of Contents**
//!
//! - [Overview](#overview)
//! - [Architecture](#architecture)
//! - [Feature Components](#feature-components)
//! - [State Management](#state-management)
//! - [Configuration System](#configuration-system)
//! - [Examples](#examples)
//!
//! ## Overview
//!
//! The UI Features System allows control over UI components using two properties:
//!
//! - **Enabled/Disabled**: Whether the feature's functionality is available
//! - **Visible/Hidden**: Whether the feature is displayed in the interface
//!
//! This design pattern allows for UI management where features can exist in three meaningful states: **Active** (enabled and visible), **Hidden** (enabled but not visible), and **Disabled** (completely inactive).
//!
//! ## Architecture
//!
//! UI Features sit at the intersection of several core Television modules, acting as a central coordination point for UI state management.
//!
//! ### Context Diagram
//!
//! ```text
//! ┌────────────┐    ┌────────────────────┐    ┌───────────────┐
//! │ CLI Module │───►│ UI Features System │◄───│ Config Module │
//! └────────────┘    └────────────────────┘    └───────────────┘
//!                             │
//!                             ▼
//!                     ┌───────────────┐
//!                     │ Screen/Layout │
//!                     │    Module     │
//!                     └───────────────┘
//!                             │
//!                             ▼
//!                       ┌───────────┐
//!                       │ UI Render │
//!                       │  System   │
//!                       └───────────┘
//! ```
//!
//! ## Feature Components
//!
//! It currently supports four primary UI features, each with distinct functionality and use cases.
//!
//! In this view you can see the `Preview`, `Help Panel`, and `Status Bar`
//!
//! ```text
//! ╭──────────────────────── Channel ─────────────────────────╮╭───────────────────────── PREVIEW ────────────────────────╮
//! │>                                                  1 / 1  ││                                                         ▲│
//! ╰──────────────────────────────────────────────────────────╯│                                                         █│
//! ╭──────────────────────── Results ─────────────────────────╮│                                                         ║│
//! │> TELEVISION                                              ││                                                         ║│
//! │                                                          ││                                                         ║│
//! │                                                          ││                                                         ║│
//! │                                                          ││                                                         ║│
//! │                                                          ││                  ╭─────────────── Help ────────────────╮║│
//! │                                                          ││                  │ Global                              │║│
//! │                                                          ││                  │ Quit: Esc                           │║│
//! │                                                          ││                  │ Quit: Ctrl-c                        │║│
//! │                                                          ││                  │ Toggle preview: Ctrl-o              │║│
//! │                                                          ││                  │ Toggle help: Ctrl-h                 │║│
//! │                                                          ││                  │ Toggle status bar: F12              │║│
//! │                                                          ││                  │                                     │║│
//! │                                                          ││                  │ Channel                             │║│
//! │                                                          ││                  │ Navigate up: Up                     │║│
//! │                                                          ││                  │ Navigate up: Ctrl-p                 │║│
//! │                                                          ││                  │ Navigate up: Ctrl-k                 │║│
//! │                                                          ││                  │ Navigate down: Down                 │║│
//! │                                                          ││                  │ Navigate down: Ctrl-n               │║│
//! │                                                          ││                  │ Navigate down: Ctrl-j               │║│
//! │                                                          ││                  │ ...                                 │║│
//! │                                                          ││                  ╰─────────────────────────────────────╯▼│
//! ╰──────────────────────────────────────────────────────────╯╰──────────────────────────────────────────────────────────╯
//!   CHANNEL custom           [Hint] Remote Control: Ctrl-t • Hide Preview: Ctrl-o • Help: Ctrl-h                  v0.00.0
//! ```
//!
//! And here you can see the `Remote Control`
//!
//! ```text
//! ╭────────────────────────────────────────────────────── Channel ───────────────────────────────────────────────────────╮
//! │>                                                                                                               1 / 1 │
//! ╰──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╯
//! ╭────────────────────────────────────────────────────── Results ───────────────────────────────────────────────────────╮
//! │> TELEVISION                                                                                                          │
//! │                                                                                                                      │
//! │                      ╭──────── Channels ─────────╮╭────── Description ───────╮                                       │
//! │                      │> alias                    ││A channel to select from  │  _____________                        │
//! │                      │                           ││shell aliases             │ /             \                       │
//! │                      │                           ││                          │ | (*)     (#) |                       │
//! │                      │                           ││                          │ |             |                       │
//! │                      │                           ││                          │ | (1) (2) (3) |                       │
//! │                      │                           ││                          │ | (4) (5) (6) |                       │
//! │                      │                           ││                          │ | (7) (8) (9) |                       │
//! │                      │                           ││                          │ |      _      |                       │
//! │                      │                           ││                          │ |     | |     |                       │
//! │                      │                           ││                          │ |  (_¯(0)¯_)  |                       │
//! │                      │                           ││                          │ |     | |     |                       │
//! │                      │                           ││                          │ |      ¯      |                       │
//! │                      │                           ││                          │ |             |                       │
//! │                      │                           ││                          │ | === === === |                       │
//! │                      ╰───────────────────────────╯╰──────────────────────────╯ |             |                       │
//! │                      ╭───────── Search ──────────╮╭─── Requirements [OK] ────╮ |     TV      |                       │
//! │                      │>                          ││                          │ `-------------´                       │
//! │                      ╰───────────────────────────╯╰──────────────────────────╯                                       │
//! │                                                                                                                      │
//! │                                                                                                                      │
//! │                                                                                                                      │
//! ╰──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╯
//!   REMOTE                              [Hint] Back to Channel: Ctrl-t • Help: Ctrl-h                             v0.00.0
//! ```
//!
//! ### Preview Panel
//!
//! Displays contextual information about the currently selected entry
//!
//! **Default State**: Enabled and Visible
//! **Configuration Files options**:
//!
//! - `size`: Width percentage (1-99)
//! - `header`: Optional template for panel header
//! - `footer`: Optional template for panel footer
//! - `scrollbar`: Whether to show scroll indicators
//!
//! **CLI Flags**: `--no-preview`, `--hide-preview`, `--show-preview`, `--preview-*` flags
//!
//! ### Status Bar
//!
//! Shows application status, mode information, and available actions
//!
//! **Default State**: Enabled and Visible
//! **Configuration**:
//!
//! - `separator_open`: Opening separator character/string
//! - `separator_close`: Closing separator character/string
//!
//! **CLI Controls**: `--no-status-bar`, `--hide-status-bar`, `--show-status-bar`
//!
//! ### Help Panel
//!
//! Displays contextual help and keyboard shortcuts
//!
//! **Default State**: Enabled but Hidden
//! **Configuration**:
//!
//! - `show_categories`: Whether to group shortcuts by category
//!
//! **CLI Controls**: `--no-help-panel`, `--hide-help-panel`, `--show-help-panel`
//!
//! ### Remote Control
//!
//! Provides channel switching and management interface
//!
//! **Default State**: Enabled but Hidden
//! **Configuration**:
//!
//! - `show_channel_descriptions`: Include channel descriptions in listing
//! - `sort_alphabetically`: Sort channels alphabetically vs. by usage
//!
//! **CLI Controls**: `--no-remote`, `--hide-remote`, `--show-remote`
//!
//! ## State Management
//!
//! Logical state transitions with are enforced with built-in constraints:
//!
//! ```text
//! ┌─────────────┐    enable()     ┌─────────────┐
//! │  Disabled   │────────────────►│   Active    │
//! │  enabled=F  │                 │  enabled=T  │
//! │  visible=F  │                 │  visible=T  │
//! └─────────────┘                 └─────────────┘
//!        ▲                               │
//!        │                               │ hide()
//!        │                               ▼
//!        │                        ┌─────────────┐
//!        └────────────────────────│   Hidden    │
//!          disable()              │  enabled=T  │
//!                                 │  visible=F  │
//!                                 └─────────────┘
//!                                        │
//!                                        │ show()
//!                                        ▼
//!                                 ┌─────────────┐
//!                                 │   Active    │
//!                                 │  enabled=T  │
//!                                 │  visible=T  │
//!                                 └─────────────┘
//! ```
//!
//! ## Configuration System
//!
//! The UI Features system configuration follows a layered priority system:
//!
//! 1. **CLI Flags** (Highest Priority)
//! 2. **Channel Configuration**
//! 3. **User Configuration File**
//! 4. **Built-in Defaults** (Lowest Priority)
//!
//! ### Configuration Formats
//!
//! **TOML Configuration Syntax**
//!
//! ```toml
//! [ui.features]
//! preview_panel = { enabled = true, visible = true }
//! help_panel = { enabled = true, visible = false }
//! status_bar = { enabled = true, visible = true }
//! remote_control = { enabled = true, visible = false }
//!
//! [ui.preview_panel]
//! size = 50
//! header = "{}"
//! footer = ""
//! scrollbar = true
//!
//! [ui.status_bar]
//! separator_open = ""
//! separator_close = ""
//!
//! [ui.remote_control]
//! show_channel_descriptions = true
//! sort_alphabetically = true
//! ```
//!
//! ### Configuration Inheritance
//!
//! **User Global Configuration**
//!
//! ```toml
//! # ~/.config/television/config.toml
//! [ui.features]
//! help_panel = { enabled = true, visible = true }  # Always show help for learning
//! ```
//!
//! **Channel-Level Configuration**
//!
//! ```toml
//! # ~/.config/television/cable/development.toml
//! [ui.features]
//! preview_panel = { enabled = true, visible = true }
//! status_bar = { enabled = true, visible = false }  # Hidden by default for focus
//! ```
//!
//! **Runtime Override Examples**
//!
//! ```bash
//! # Override channel defaults
//! tv development --show-status-bar --hide-preview
//!
//! # Force features on/off
//! tv files --no-remote --show-help-panel
//!
//! # Mixed visibility control
//! tv git-log --hide-status-bar --show-preview
//! ```
//!
//! ### Default UI Feature States
//!
//! | UI Feature | Default Enabled | Default Visible | Rationale |
//! |------------|----------------|-----------------|-----------|
//! | **Preview Panel** | ✅ | ✅ | Core functionality |
//! | **Status Bar** | ✅ | ✅ | Shows mode and contextual hint to the user |
//! | **Help Panel** | ✅ | ❌ | Available on-demand to avoid clutter |
//! | **Remote Control** | ✅ | ❌ | Available on-demand, disrupts regular operation |
//!
//! ### Feature State Persistence
//!
//! **What Persists Across Sessions**
//!
//! - ✅ **Configuration file settings** - Feature states defined in `~/.config/television/config.toml`
//! - ✅ **Channel-specific defaults** - Feature configurations built into channel definitions
//!
//! **What Does Not Persist**
//!
//! - ❌ **Runtime toggles** - Keyboard shortcuts like `Tab` (preview) or `F2` (status bar) are session-only
//! - ❌ **CLI flag overrides** - `--hide-preview`, `--show-status-bar` etc. apply only to current session
//! - ❌ **Temporary state changes** - Any feature visibility changes made during application use
//!
//! ## Examples
//!
//! ### Basic Feature Control
//!
//! **Hide Preview Panel**
//!
//! ```bash
//! tv files --hide-preview
//! ```
//!
//! **Disable All Optional Features**
//!
//! ```bash
//! tv files --no-preview --no-status-bar --no-remote --no-help-panel
//! ```
//!
//! **Show All Features**
//!
//! ```bash
//! tv files --show-preview --show-status-bar --show-help-panel
//! ```
//!
//! ### Channel-Specific Configuration
//!
//! **Create Development Channel with Custom Features**
//!
//! ```toml
//! # ~/.config/television/cable/dev-focused.toml
//! [ui.features]
//! preview_panel = { enabled = true, visible = true }
//! status_bar = { enabled = true, visible = false }    # Clean interface
//! help_panel = { enabled = true, visible = false }    # Help on-demand
//! remote_control = { enabled = false }                # Single-channel focus
//! ```
//!
//! **Usage**
//!
//! ```bash
//! tv dev-focused /path/to/project
//! ```
//!
//! ### Runtime Feature Management
//!
//! **Quick Interface Cleanup**
//!
//! ```bash
//! # Start with full interface
//! tv files
//!
//! # Runtime toggles (using default keybindings):
//! # Ctrl+O - Toggle preview panel
//! # F12    - Toggle status bar
//! # Ctrl-H - Toggle help panel
//! ```

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// Represents the state of a feature in the UI, namely whether it is enabled and visible.
///
/// TODO: This could be a good candidate for a typestate but let's keep it simple for now.
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

    /// Toggles the enabled state of the feature and updates visibility accordingly.
    pub fn toggle_enabled(&mut self) {
        self.enabled = !self.enabled;
        // If disabling, also hide
        if !self.enabled {
            self.visible = false;
        }
    }

    /// Toggles the visibility of the feature.
    ///
    /// This has no effect if the feature is disabled.
    pub fn toggle_visible(&mut self) {
        if self.enabled {
            self.visible = !self.visible;
        }
    }

    /// Enables the feature, making it both enabled and visible.
    pub fn enable(&mut self) {
        self.enabled = true;
        self.visible = true;
    }

    /// Disables the feature, making it both disabled and hidden.
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
/// Represents the collection of features available in the UI, each with its own state.
///
/// This currently defaults to the following:
/// - `preview_panel`: Enabled and Visible
/// - `help_panel`: Enabled but Hidden
/// - `status_bar`: Enabled and Visible
/// - `remote_control`: Enabled but Hidden
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
#[serde(rename_all = "snake_case")]
/// This enum defines the available feature flags in the UI Features System.
///
/// It is used throughout the application to reference specific features and get/set their state.
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
//
// This is used to convert the `Features` struct to and from the cable channel configuration
// format.
impl Serialize for Features {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeMap;

        let mut map = serializer.serialize_map(Some(4))?;
        map.serialize_entry(&FeatureFlags::PreviewPanel, &self.preview_panel)?;
        map.serialize_entry(&FeatureFlags::HelpPanel, &self.help_panel)?;
        map.serialize_entry(&FeatureFlags::StatusBar, &self.status_bar)?;
        map.serialize_entry(
            &FeatureFlags::RemoteControl,
            &self.remote_control,
        )?;
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

                while let Some(key) = map.next_key::<FeatureFlags>()? {
                    match key {
                        FeatureFlags::PreviewPanel => {
                            features.preview_panel = map.next_value()?;
                        }
                        FeatureFlags::HelpPanel => {
                            features.help_panel = map.next_value()?;
                        }
                        FeatureFlags::StatusBar => {
                            features.status_bar = map.next_value()?;
                        }
                        FeatureFlags::RemoteControl => {
                            features.remote_control = map.next_value()?;
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
