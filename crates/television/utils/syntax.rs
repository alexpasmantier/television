use std::path::Path;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, Theme};
use syntect::parsing::SyntaxSet;
use tracing::warn;

pub fn compute_highlights_for_path(
    file_path: &Path,
    lines: Vec<String>,
    syntax_set: &SyntaxSet,
    syntax_theme: &Theme,
) -> color_eyre::Result<Vec<Vec<(Style, String)>>> {
    let syntax =
        syntax_set
            .find_syntax_for_file(file_path)?
            .unwrap_or_else(|| {
                warn!(
                    "No syntax found for {:?}, defaulting to plain text",
                    file_path
                );
                syntax_set.find_syntax_plain_text()
            });
    let mut highlighter = HighlightLines::new(syntax, syntax_theme);
    let mut highlighted_lines = Vec::new();
    for line in lines {
        let hl_regions = highlighter.highlight_line(&line, syntax_set)?;
        highlighted_lines.push(
            hl_regions
                .iter()
                .map(|(style, text)| (*style, (*text).to_string()))
                .collect(),
        );
    }
    Ok(highlighted_lines)
}

pub fn compute_highlights_for_line<'a>(
    line: &'a str,
    syntax_set: &SyntaxSet,
    syntax_theme: &Theme,
    file_path: &str,
) -> color_eyre::Result<Vec<(Style, &'a str)>> {
    let syntax = syntax_set.find_syntax_for_file(file_path)?;
    match syntax {
        None => {
            warn!(
                "No syntax found for path {:?}, defaulting to plain text",
                file_path
            );
            Ok(vec![(Style::default(), line)])
        }
        Some(syntax) => {
            let mut highlighter = HighlightLines::new(syntax, syntax_theme);
            Ok(highlighter.highlight_line(line, syntax_set)?)
        }
    }
}
