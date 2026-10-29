//! For ansi styled inputs, we need to store both the stripped text (for the matcher)
//! and the style for rendering.
//!
//! The styles are broken down into:
//! - a style table of the distinct styles seen in the channel's output
//! - a list of (starting offset, style id in the style table) for each line
//!
//! This is mostly a space optimization.

use parking_lot::RwLock;
use ratatui::style::{Color, Modifier, Style};
use rustc_hash::FxHashMap;
use smallvec::SmallVec;
use std::sync::Arc;
use vt_push_parser::{VT_PARSER_INTEREST_CSI, VTPushParser, event::VTEvent};

/// A start offset (in chars) and a style id (that maps to a style in the style table).
pub type StyleRun = (u32, u16);

/// The style runs for a line.
pub type StyleRuns = SmallVec<[StyleRun; 5]>;

/// This keeps track of the distinct styles seen in a channel's output and maps each one to a small
/// integer id (its index in the `styles` vector).
#[derive(Debug, Default)]
pub struct StyleTable {
    styles: Vec<Style>,
    ids_by_style: FxHashMap<Style, u16>,
}

impl StyleTable {
    /// Add a style to the table if it isn't already present, returning its id.
    fn intern(&mut self, style: Style) -> u16 {
        if let Some(&id) = self.ids_by_style.get(&style) {
            return id;
        }
        let id = u16::try_from(self.styles.len()).unwrap_or(u16::MAX);
        if usize::from(id) == self.styles.len() {
            self.styles.push(style);
            self.ids_by_style.insert(style, id);
        }
        id
    }

    /// The style registered under `id`, or the default style if unknown.
    pub fn resolve(&self, id: u16) -> Style {
        self.styles
            .get(usize::from(id))
            .copied()
            .unwrap_or_default()
    }
}

/// A handle to the style table (shared by the ingest workers and the screen to render results).
pub type SharedStyleTable = Arc<RwLock<StyleTable>>;

#[derive(Debug, Clone)]
pub struct AnsiParser {
    style_table: SharedStyleTable,
    /// Caching the last seen styles locally is a cheap way to avoid locks on the shared style table
    cache: Vec<(Style, u16)>,
}

impl AnsiParser {
    pub fn new(style_table: SharedStyleTable) -> Self {
        Self {
            style_table,
            cache: Vec::new(),
        }
    }

    fn intern(&mut self, style: Style) -> u16 {
        if let Some(&(_, id)) =
            self.cache.iter().find(|(cached, _)| *cached == style)
        {
            return id;
        }
        let id = self.style_table.write().intern(style);
        self.cache.push((style, id));
        id
    }

    /// Split a line into its stripped text and the style runs over it.
    #[allow(clippy::cast_possible_truncation)]
    pub fn parse(&mut self, line: &str) -> (String, StyleRuns) {
        let mut text = String::with_capacity(line.len());
        let mut runs = StyleRuns::new();
        let mut style = Style::default();
        let mut last_style = Style::default();
        let mut chars: u32 = 0;

        let mut parser =
            VTPushParser::new_with_interest::<VT_PARSER_INTEREST_CSI>();
        parser.feed_with(line.as_bytes(), |event: VTEvent| match event {
            VTEvent::Raw(bytes) => {
                let chunk = String::from_utf8_lossy(bytes);
                if chunk.is_empty() {
                    return;
                }
                if style != last_style {
                    runs.push((chars, self.intern(style)));
                    last_style = style;
                }
                chars += chunk.chars().count() as u32;
                text.push_str(&chunk);
            }
            // `m` is SGR, the only sequence that carries styling
            VTEvent::Csi(csi) if csi.final_byte == b'm' => {
                style = apply_sgr(style, &csi);
            }
            _ => {}
        });

        text.shrink_to_fit();
        (text, runs)
    }
}

/// Apply an SGR sequence (italic, bold, ...) to a style.
fn apply_sgr(mut style: Style, csi: &vt_push_parser::event::CSI<'_>) -> Style {
    let params: Vec<u16> = csi
        .params
        .into_iter()
        .map(|p| {
            std::str::from_utf8(p)
                .ok()
                .and_then(|s| s.parse::<u16>().ok())
                .unwrap_or(0)
        })
        .collect();
    // An SGR with no parameters at all (`ESC[m`) is a reset
    if params.is_empty() {
        return Style::default();
    }

    let mut i = 0;
    while i < params.len() {
        match params[i] {
            0 => style = Style::default(),
            1 => style = style.add_modifier(Modifier::BOLD),
            2 => style = style.add_modifier(Modifier::DIM),
            3 => style = style.add_modifier(Modifier::ITALIC),
            4 => style = style.add_modifier(Modifier::UNDERLINED),
            5 | 6 => style = style.add_modifier(Modifier::SLOW_BLINK),
            7 => style = style.add_modifier(Modifier::REVERSED),
            8 => style = style.add_modifier(Modifier::HIDDEN),
            9 => style = style.add_modifier(Modifier::CROSSED_OUT),
            22 => {
                style = style.remove_modifier(Modifier::BOLD | Modifier::DIM);
            }
            23 => style = style.remove_modifier(Modifier::ITALIC),
            24 => style = style.remove_modifier(Modifier::UNDERLINED),
            25 => style = style.remove_modifier(Modifier::SLOW_BLINK),
            27 => style = style.remove_modifier(Modifier::REVERSED),
            28 => style = style.remove_modifier(Modifier::HIDDEN),
            29 => style = style.remove_modifier(Modifier::CROSSED_OUT),
            c @ 30..=37 => style = style.fg(ansi_color(c - 30)),
            38 => {
                if let Some((color, consumed)) = extended_color(&params[i..]) {
                    style = style.fg(color);
                    i += consumed;
                    continue;
                }
            }
            39 => style = style.fg(Color::Reset),
            c @ 40..=47 => style = style.bg(ansi_color(c - 40)),
            48 => {
                if let Some((color, consumed)) = extended_color(&params[i..]) {
                    style = style.bg(color);
                    i += consumed;
                    continue;
                }
            }
            49 => style = style.bg(Color::Reset),
            c @ 90..=97 => style = style.fg(bright_color(c - 90)),
            c @ 100..=107 => style = style.bg(bright_color(c - 100)),
            // ignore unknown codes
            _ => {}
        }
        i += 1;
    }
    style
}

/// Decode a `38`/`48` extended color, returning it with the number of
/// parameters it consumed.
fn extended_color(params: &[u16]) -> Option<(Color, usize)> {
    match params.get(1)? {
        // 5;n -> 256-color palette
        5 => {
            let n = *params.get(2)?;
            Some((Color::Indexed(u8::try_from(n).ok()?), 3))
        }
        // 2;r;g;b -> truecolor
        2 => {
            let r = u8::try_from(*params.get(2)?).ok()?;
            let g = u8::try_from(*params.get(3)?).ok()?;
            let b = u8::try_from(*params.get(4)?).ok()?;
            Some((Color::Rgb(r, g, b), 5))
        }
        _ => None,
    }
}

const fn ansi_color(n: u16) -> Color {
    match n {
        0 => Color::Black,
        1 => Color::Red,
        2 => Color::Green,
        3 => Color::Yellow,
        4 => Color::Blue,
        5 => Color::Magenta,
        6 => Color::Cyan,
        _ => Color::Gray,
    }
}

const fn bright_color(n: u16) -> Color {
    match n {
        0 => Color::DarkGray,
        1 => Color::LightRed,
        2 => Color::LightGreen,
        3 => Color::LightYellow,
        4 => Color::LightBlue,
        5 => Color::LightMagenta,
        6 => Color::LightCyan,
        _ => Color::White,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fast_strip_ansi::strip_ansi_string;
    use std::fmt::Write;

    fn parser() -> AnsiParser {
        AnsiParser::new(Arc::new(RwLock::new(StyleTable::default())))
    }

    /// The line `rg` emits for the text channel: path, line number and
    /// content, each in its own color.
    const RG_LINE: &str = "\x1b[0m\x1b[34mKconfig\x1b[0m:\x1b[0m\x1b[32m12\x1b[0m:\x1b[0m\x1b[1m\x1b[37m# a comment\x1b[0m";

    #[test]
    fn stripped_text_matches_the_standalone_stripper() {
        let mut p = parser();
        for line in [
            RG_LINE,
            "no ansi here",
            "\x1b[31mred\x1b[0m and \x1b[1;32mbold green\x1b[0m",
            "",
            "\x1b[38;2;255;0;0mtruecolor\x1b[0m",
        ] {
            let (text, _) = p.parse(line);
            assert_eq!(text, strip_ansi_string(line), "line: {line:?}");
        }
    }

    #[test]
    fn rg_line_yields_one_run_per_colored_field() {
        let mut p = parser();
        let (text, runs) = p.parse(RG_LINE);
        assert_eq!(text, "Kconfig:12:# a comment");

        let style_table = p.style_table.read();
        let styles: Vec<(u32, Style)> = runs
            .iter()
            .map(|&(at, id)| (at, style_table.resolve(id)))
            .collect();
        assert_eq!(
            styles,
            vec![
                (0, Style::default().fg(Color::Blue)),
                (7, Style::default()),
                (8, Style::default().fg(Color::Green)),
                (10, Style::default()),
                (
                    11,
                    Style::default()
                        .fg(Color::Gray)
                        .add_modifier(Modifier::BOLD)
                ),
            ]
        );
    }

    #[test]
    fn unstyled_lines_carry_no_runs() {
        let mut p = parser();
        assert!(p.parse("plain text").1.is_empty());
        // styling that resets to default before any text counts as unstyled
        assert!(p.parse("\x1b[0mplain text").1.is_empty());
    }

    /// Offsets index characters (lines up with match indices).
    #[test]
    fn offsets_are_character_based() {
        let mut p = parser();
        // "héllo" is 5 characters but 6 bytes
        let (text, runs) = p.parse("héllo\x1b[31mworld\x1b[0m");
        assert_eq!(text, "hélloworld");
        assert_eq!(runs.last().unwrap().0, 5);
    }

    #[test]
    fn identical_styles_are_interned_once() {
        let mut p = parser();
        p.parse("\x1b[31mred\x1b[0m");
        p.parse("\x1b[31malso red\x1b[0m");
        // red, registered once
        assert_eq!(p.style_table.read().styles.len(), 1);
    }

    #[test]
    fn runs_stay_inline_for_rg_output() {
        let mut p = parser();
        let (_, runs) = p.parse(RG_LINE);
        assert!(!runs.spilled(), "rg lines should not allocate");
    }

    /// Styling that changes on every character (e.g. a truecolor gradient)
    /// still parses correctly (the runs spill to the heap).
    #[test]
    fn heavily_styled_lines_still_parse() {
        let mut p = parser();
        let mut line = String::new();
        for i in 0..64u8 {
            let _ = write!(line, "\x1b[38;2;{i};0;0mx");
        }
        let (text, runs) = p.parse(&line);
        assert_eq!(text, "x".repeat(64));
        assert_eq!(runs.len(), 64);
        assert_eq!(p.style_table.read().styles.len(), 64);
    }
}
