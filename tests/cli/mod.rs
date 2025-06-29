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

#[path = "../common/mod.rs"]
mod common;

pub mod cli_config;
pub mod cli_errors;
pub mod cli_input;
pub mod cli_modes;
pub mod cli_monitoring;
pub mod cli_preview;
pub mod cli_selection;
pub mod cli_source;
pub mod cli_special;
pub mod cli_ui;
pub mod cli_ui_behavior;
