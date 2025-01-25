use anyhow::Result;
use bat::assets::HighlightingAssets;
use gag::Gag;
use std::path::{Path, PathBuf};
use syntect::easy::HighlightLines;
use syntect::highlighting::{
    HighlightIterator, HighlightState, Highlighter, Style, Theme,
};
use syntect::parsing::{ParseState, ScopeStack, SyntaxReference, SyntaxSet};
use tracing::warn;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HighlightingState {
    parse_state: ParseState,
    highlight_state: HighlightState,
}

impl HighlightingState {
    pub fn new(
        parse_state: ParseState,
        highlight_state: HighlightState,
    ) -> Self {
        Self {
            parse_state,
            highlight_state,
        }
    }
}

struct LineHighlighter<'a> {
    highlighter: Highlighter<'a>,
    pub parse_state: ParseState,
    pub highlight_state: HighlightState,
}

impl<'a> LineHighlighter<'a> {
    pub fn new(
        syntax: &SyntaxReference,
        theme: &'a Theme,
    ) -> LineHighlighter<'a> {
        let highlighter = Highlighter::new(theme);
        let highlight_state =
            HighlightState::new(&highlighter, ScopeStack::new());
        Self {
            highlighter,
            parse_state: ParseState::new(syntax),
            highlight_state,
        }
    }

    #[allow(dead_code)]
    pub fn from_state(
        state: HighlightingState,
        theme: &'a Theme,
    ) -> LineHighlighter<'a> {
        Self {
            highlighter: Highlighter::new(theme),
            parse_state: state.parse_state,
            highlight_state: state.highlight_state,
        }
    }

    /// Highlights a line of a file
    pub fn highlight_line<'b>(
        &mut self,
        line: &'b str,
        syntax_set: &SyntaxSet,
    ) -> Result<Vec<(Style, &'b str)>, syntect::Error> {
        let ops = self.parse_state.parse_line(line, syntax_set)?;
        let iter = HighlightIterator::new(
            &mut self.highlight_state,
            &ops[..],
            line,
            &self.highlighter,
        );
        Ok(iter.collect())
    }
}

#[deprecated(
    note = "Use `compute_highlights_incremental` instead, which also returns the state"
)]
pub fn compute_highlights_for_path(
    file_path: &Path,
    lines: &[String],
    syntax_set: &SyntaxSet,
    syntax_theme: &Theme,
) -> Result<Vec<Vec<(Style, String)>>> {
    let syntax = set_syntax_set(syntax_set, file_path);
    let mut highlighter = HighlightLines::new(syntax, syntax_theme);
    let mut highlighted_lines = Vec::new();
    for line in lines {
        let hl_regions = highlighter.highlight_line(line, syntax_set)?;
        highlighted_lines.push(
            hl_regions
                .iter()
                .map(|(style, text)| (*style, (*text).to_string()))
                .collect(),
        );
    }
    Ok(highlighted_lines)
}

fn set_syntax_set<'a>(
    syntax_set: &'a SyntaxSet,
    file_path: &Path,
) -> &'a SyntaxReference {
    syntax_set
        .find_syntax_for_file(file_path)
        .unwrap_or(None)
        .unwrap_or_else(|| {
            warn!(
                "No syntax found for {:?}, defaulting to plain text",
                file_path
            );
            syntax_set.find_syntax_plain_text()
        })
}

#[derive(Debug, Clone)]
pub struct HighlightedLines {
    pub lines: Vec<Vec<(Style, String)>>,
    //pub state: Option<HighlightingState>,
}

impl HighlightedLines {
    pub fn new(
        lines: Vec<Vec<(Style, String)>>,
        _state: &Option<HighlightingState>,
    ) -> Self {
        Self { lines, /*state*/ }
    }
}

pub fn compute_highlights_incremental(
    file_path: &Path,
    lines: &[String],
    syntax_set: &SyntaxSet,
    syntax_theme: &Theme,
    _cached_lines: &Option<HighlightedLines>,
) -> Result<HighlightedLines> {
    let mut highlighted_lines: Vec<_>;
    let mut highlighter: LineHighlighter;

    //if let Some(HighlightedLines {
    //    lines: c_lines,
    //    state: Some(s),
    //}) = cached_lines
    //{
    //    highlighter = LineHighlighter::from_state(s, syntax_theme);
    //    highlighted_lines = c_lines;
    //} else {
    //    let syntax = set_syntax_set(syntax_set, file_path);
    //    highlighter = LineHighlighter::new(syntax, syntax_theme);
    //    highlighted_lines = Vec::new();
    //};
    let syntax = set_syntax_set(syntax_set, file_path);
    highlighter = LineHighlighter::new(syntax, syntax_theme);
    highlighted_lines = Vec::with_capacity(lines.len());

    for line in lines {
        let hl_regions = highlighter.highlight_line(line, syntax_set)?;
        highlighted_lines.push(
            hl_regions
                .iter()
                .map(|(style, text)| (*style, (*text).to_string()))
                .collect(),
        );
    }

    Ok(HighlightedLines::new(
        highlighted_lines,
        &Some(HighlightingState::new(
            highlighter.parse_state.clone(),
            highlighter.highlight_state.clone(),
        )),
    ))
}

#[allow(dead_code)]
pub fn compute_highlights_for_line<'a>(
    line: &'a str,
    syntax_set: &SyntaxSet,
    syntax_theme: &Theme,
    file_path: &str,
) -> Result<Vec<(Style, &'a str)>> {
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

// Based on code from https://github.com/sharkdp/bat e981e974076a926a38f124b7d8746de2ca5f0a28
//
// Copyright (c) 2018-2023 bat-developers (https://github.com/sharkdp/bat).
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use directories::BaseDirs;
use lazy_static::lazy_static;

#[cfg(target_os = "macos")]
use std::env;

/// Wrapper for 'dirs' that treats `MacOS` more like `Linux`, by following the XDG specification.
///
/// This means that the `XDG_CACHE_HOME` and `XDG_CONFIG_HOME` environment variables are
/// checked first. The fallback directories are `~/.cache/bat` and `~/.config/bat`, respectively.
pub struct BatProjectDirs {
    cache_dir: PathBuf,
}

impl BatProjectDirs {
    fn new() -> Option<BatProjectDirs> {
        #[cfg(target_os = "macos")]
        let cache_dir_op = env::var_os("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
            .or_else(|| BaseDirs::new().map(|d| d.home_dir().join(".cache")));

        #[cfg(not(target_os = "macos"))]
        let cache_dir_op = BaseDirs::new().map(|d| d.cache_dir().to_owned());

        let cache_dir = cache_dir_op.map(|d| d.join("bat"))?;

        Some(BatProjectDirs { cache_dir })
    }

    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }
}

lazy_static! {
    pub static ref PROJECT_DIRS: BatProjectDirs = BatProjectDirs::new()
        .unwrap_or_else(|| panic!("Could not get home directory"));
}

pub fn load_highlighting_assets() -> HighlightingAssets {
    HighlightingAssets::from_cache(PROJECT_DIRS.cache_dir())
        .unwrap_or_else(|_| HighlightingAssets::from_binary())
}

pub trait HighlightingAssetsExt {
    fn get_theme_no_output(&self, theme_name: &str) -> &Theme;
}

impl HighlightingAssetsExt for HighlightingAssets {
    /// Get a theme by name. If the theme is not found, the default theme is returned.
    ///
    /// This is an ugly hack to work around the fact that bat actually prints a warning
    /// to stderr when a theme is not found which might mess up the TUI. This function
    /// suppresses that warning by temporarily redirecting stderr and stdout.
    fn get_theme_no_output(&self, theme_name: &str) -> &Theme {
        let _e = Gag::stderr().unwrap();
        let _o = Gag::stdout().unwrap();
        let theme = self.get_theme(theme_name);
        theme
    }
}
