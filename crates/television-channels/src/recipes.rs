use std::{collections::HashMap, ops::Deref};

const RECIPES: &str = include_str!("../../../.config/recipes.toml");

#[derive(Clone, Debug, serde::Deserialize)]
pub struct ChannelRecipe {
    pub name: String,
    pub source_command: String,
    pub preview_command: String,
    #[serde(default = "default_delimiter")]
    pub preview_delimiter: String,
}

const DEFAULT_DELIMITER: &str = " ";

fn default_delimiter() -> String {
    DEFAULT_DELIMITER.to_string()
}

/// Just a proxy struct to deserialize the recipes from the TOML file.
#[derive(Debug, serde::Deserialize)]
struct Recipes {
    recipes: Vec<ChannelRecipe>,
}

impl ToString for ChannelRecipe {
    fn to_string(&self) -> String {
        self.name.clone()
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct ChannelCookBook(pub HashMap<String, ChannelRecipe>);

impl Deref for ChannelCookBook {
    type Target = HashMap<String, ChannelRecipe>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn load_cook_book() -> Result<ChannelCookBook, toml::de::Error> {
    let r: Recipes = toml::from_str(RECIPES)?;
    let mut cook_book = HashMap::new();
    for recipe in r.recipes {
        cook_book.insert(recipe.name.clone(), recipe);
    }
    Ok(ChannelCookBook(cook_book))
}
