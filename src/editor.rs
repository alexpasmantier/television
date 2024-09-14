use std::env;
use std::env::VarError;

const DEFAULT_EDITOR: &str = "vi";

/// Get the default editor for the current environment
pub fn get_default_editor() -> Result<String, VarError> {
    match env::var("VISUAL") {
        Ok(result) => return Ok(result),
        Err(VarError::NotPresent) => {}
        Err(error) => return Err(error),
    }

    match env::var("EDITOR") {
        Ok(result) => return Ok(result),
        Err(VarError::NotPresent) => {}
        Err(error) => return Err(error),
    }

    Ok(DEFAULT_EDITOR.to_string())
}
