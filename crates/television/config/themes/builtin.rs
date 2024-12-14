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
            "catppuccin-mocha",
            include_str!("../../../../themes/catppuccin-mocha.toml"),
        );
        m.insert(
            "nord-dark",
            include_str!("../../../../themes/nord-dark.toml"),
        );
        m.insert(
            "solarized-dark",
            include_str!("../../../../themes/solarized-dark.toml"),
        );
        m
    };
}
