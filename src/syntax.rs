use ratatui::{
    style::{Style, Stylize},
    text::Span,
};
use syntect;

pub fn convert_syn_region_to_span<'a>(
    syn_region: (syntect::highlighting::Style, String),
    background: Option<syntect::highlighting::Color>,
) -> Span<'a> {
    let mut style =
        Style::default().fg(convert_syn_color_to_ratatui_color(syn_region.0.foreground));
    if let Some(background) = background {
        style = style.bg(convert_syn_color_to_ratatui_color(background));
    }
    style = match syn_region.0.font_style {
        syntect::highlighting::FontStyle::BOLD => style.bold(),
        syntect::highlighting::FontStyle::ITALIC => style.italic(),
        syntect::highlighting::FontStyle::UNDERLINE => style.underlined(),
        _ => style,
    };
    Span::styled(syn_region.1.clone(), style)
}

pub fn convert_syn_color_to_ratatui_color(
    color: syntect::highlighting::Color,
) -> ratatui::style::Color {
    ratatui::style::Color::Rgb(color.r, color.g, color.b)
}

pub struct RThemeSettings {
    /// The default color for text.
    pub foreground: Option<ratatui::style::Color>,
    /// The default backgound color of the view.
    pub background: Option<ratatui::style::Color>,
    /// ratatui::style::Color of the caret.
    pub caret: Option<ratatui::style::Color>,
    /// ratatui::style::Color of the line the caret is in.
    /// Only used when the `highlight_line` setting is set to `true`.
    pub line_highlight: Option<ratatui::style::Color>,

    /// The color to use for the squiggly underline drawn under misspelled words.
    pub misspelling: Option<ratatui::style::Color>,
    /// The color of the border drawn around the viewport area of the minimap.
    /// Only used when the `draw_minimap_border` setting is enabled.
    pub minimap_border: Option<ratatui::style::Color>,
    /// A color made available for use by the theme.
    pub accent: Option<ratatui::style::Color>,

    /// ratatui::style::Color of bracketed sections of text when the caret is in a bracketed section.
    /// Only applied when the `match_brackets` setting is set to `true`.
    pub bracket_contents_foreground: Option<ratatui::style::Color>,
    /// Foreground color of the brackets when the caret is next to a bracket.
    /// Only applied when the `match_brackets` setting is set to `true`.
    pub brackets_foreground: Option<ratatui::style::Color>,
    /// Background color of the brackets when the caret is next to a bracket.
    /// Only applied when the `match_brackets` setting is set to `true`.
    pub brackets_background: Option<ratatui::style::Color>,

    /// ratatui::style::Color of tags when the caret is next to a tag.
    /// Only used when the `match_tags` setting is set to `true`.
    pub tags_foreground: Option<ratatui::style::Color>,

    /// The border color for "other" matches.
    pub highlight: Option<ratatui::style::Color>,
    /// Background color of regions matching the current search.
    pub find_highlight: Option<ratatui::style::Color>,
    /// Text color of regions matching the current search.
    pub find_highlight_foreground: Option<ratatui::style::Color>,

    /// Background color of the gutter.
    pub gutter: Option<ratatui::style::Color>,
    /// Foreground color of the gutter.
    pub gutter_foreground: Option<ratatui::style::Color>,

    /// The background color of selected text.
    pub selection: Option<ratatui::style::Color>,
    /// A color that will override the scope-based text color of the selection.
    pub selection_foreground: Option<ratatui::style::Color>,

    /// ratatui::style::Color of the selection regions border.
    pub selection_border: Option<ratatui::style::Color>,
    /// The background color of a selection in a view that is not currently focused.
    pub inactive_selection: Option<ratatui::style::Color>,
    /// A color that will override the scope-based text color of the selection
    /// in a view that is not currently focused.
    pub inactive_selection_foreground: Option<ratatui::style::Color>,

    /// ratatui::style::Color of the guides displayed to indicate nesting levels.
    pub guide: Option<ratatui::style::Color>,
    /// ratatui::style::Color of the guide lined up with the caret.
    /// Only applied if the `indent_guide_options` setting is set to `draw_active`.
    pub active_guide: Option<ratatui::style::Color>,
    /// ratatui::style::Color of the current guide’s parent guide level.
    /// Only used if the `indent_guide_options` setting is set to `draw_active`.
    pub stack_guide: Option<ratatui::style::Color>,

    /// The color of the shadow used when a text area can be horizontally scrolled.
    pub shadow: Option<ratatui::style::Color>,
}

impl From<syntect::highlighting::ThemeSettings> for RThemeSettings {
    fn from(settings: syntect::highlighting::ThemeSettings) -> Self {
        Self {
            foreground: settings.foreground.map(convert_syn_color_to_ratatui_color),
            background: settings.background.map(convert_syn_color_to_ratatui_color),
            caret: settings.caret.map(convert_syn_color_to_ratatui_color),
            line_highlight: settings
                .line_highlight
                .map(convert_syn_color_to_ratatui_color),
            misspelling: settings.misspelling.map(convert_syn_color_to_ratatui_color),
            minimap_border: settings
                .minimap_border
                .map(convert_syn_color_to_ratatui_color),
            accent: settings.accent.map(convert_syn_color_to_ratatui_color),
            bracket_contents_foreground: settings
                .bracket_contents_foreground
                .map(convert_syn_color_to_ratatui_color),
            brackets_foreground: settings
                .brackets_foreground
                .map(convert_syn_color_to_ratatui_color),
            brackets_background: settings
                .brackets_background
                .map(convert_syn_color_to_ratatui_color),
            tags_foreground: settings
                .tags_foreground
                .map(convert_syn_color_to_ratatui_color),
            highlight: settings.highlight.map(convert_syn_color_to_ratatui_color),
            find_highlight: settings
                .find_highlight
                .map(convert_syn_color_to_ratatui_color),
            find_highlight_foreground: settings
                .find_highlight_foreground
                .map(convert_syn_color_to_ratatui_color),
            gutter: settings.gutter.map(convert_syn_color_to_ratatui_color),
            gutter_foreground: settings
                .gutter_foreground
                .map(convert_syn_color_to_ratatui_color),
            selection: settings.selection.map(convert_syn_color_to_ratatui_color),
            selection_foreground: settings
                .selection_foreground
                .map(convert_syn_color_to_ratatui_color),
            selection_border: settings
                .selection_border
                .map(convert_syn_color_to_ratatui_color),
            inactive_selection: settings
                .inactive_selection
                .map(convert_syn_color_to_ratatui_color),
            inactive_selection_foreground: settings
                .inactive_selection_foreground
                .map(convert_syn_color_to_ratatui_color),
            guide: settings.guide.map(convert_syn_color_to_ratatui_color),
            active_guide: settings
                .active_guide
                .map(convert_syn_color_to_ratatui_color),
            stack_guide: settings.stack_guide.map(convert_syn_color_to_ratatui_color),
            shadow: settings.shadow.map(convert_syn_color_to_ratatui_color),
        }
    }
}
