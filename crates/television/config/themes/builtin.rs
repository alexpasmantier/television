use std::collections::HashMap;

use lazy_static::lazy_static;

lazy_static! {
    pub static ref BUILTIN_THEMES: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert(
            "gruvbox-dark",
            include_str!("../../../../themes/gruvbox-dark.toml"),
        );
        m.insert(
            "catppuccin",
            include_str!("../../../../themes/catppuccin.toml"),
        );
        m.insert("nord", include_str!("../../../../themes/nord.toml"));
        m
    };
}
