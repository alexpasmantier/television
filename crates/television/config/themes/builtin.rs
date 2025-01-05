use std::collections::HashMap;

use lazy_static::lazy_static;

lazy_static! {
    pub static ref BUILTIN_THEMES: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert(
            "television",
            include_str!("../../../../themes/television.toml"),
        );
        m.insert(
            "gruvbox-dark",
            include_str!("../../../../themes/gruvbox-dark.toml"),
        );
        m.insert(
            "gruvbox-light",
            include_str!("../../../../themes/gruvbox-light.toml"),
        );
        m.insert(
            "catppuccin",
            include_str!("../../../../themes/catppuccin.toml"),
        );
        m.insert(
            "nord-dark",
            include_str!("../../../../themes/nord-dark.toml"),
        );
        m.insert(
            "solarized-dark",
            include_str!("../../../../themes/solarized-dark.toml"),
        );
        m.insert(
            "solarized-light",
            include_str!("../../../../themes/solarized-light.toml"),
        );
        m.insert("dracula", include_str!("../../../../themes/dracula.toml"));
        m.insert("monokai", include_str!("../../../../themes/monokai.toml"));
        m.insert("onedark", include_str!("../../../../themes/onedark.toml"));
        m.insert(
            "tokyonight",
            include_str!("../../../../themes/tokyonight.toml"),
        );
        m
    };
}
