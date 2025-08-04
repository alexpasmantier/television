//! CLI integration tests for Television
//!
//! These tests verify the command-line interface functionality, including:
//! - Configuration and directory handling
//! - Input and interaction options
//! - Operating modes and path detection
//! - Performance and monitoring features
//! - Preview functionality
//! - Selection behaviors
//! - Source command handling
//! - Special modes and shell integration
//! - UI customization and layout
//! - UI behavioral integration
//! - Error handling and validation

pub mod config;
pub mod errors;
pub mod external_actions;
pub mod input;
pub mod modes;
pub mod monitoring;
pub mod preview;
pub mod selection;
pub mod source;
pub mod special;
pub mod ui;
pub mod ui_behavior;
