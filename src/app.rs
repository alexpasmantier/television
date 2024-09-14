use std::collections::HashMap;
use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::u16;

use crossbeam::queue::SegQueue;
use ratatui::widgets::ListState;
use syntect::easy::HighlightFile;
use syntect::highlighting::{Style, Theme};
use syntect::parsing::SyntaxSet;
use tracing::debug;

use crate::ui::Input;

use crate::syntax::RThemeSettings;

pub const DEFAULT_PATH: &str = ".";

#[derive(Clone, Copy, PartialEq)]
pub enum CurrentBlock {
    Search,
    Results,
    Preview,
}

static BLOCKS: [CurrentBlock; 3] = [
    CurrentBlock::Search,
    CurrentBlock::Results,
    CurrentBlock::Preview,
];

#[derive(Clone, Debug)]
pub struct Result {
    pub path: PathBuf,
    pub line_number: usize,
    pub line: String,
    //pub matches: Vec<MatchRange>,
}

pub fn file_results_to_ui_results(file_result: FileResults) -> Vec<Result> {
    let path = file_result.path;
    file_result
        .results
        .iter()
        //.map(|r: &SearchResult| Result {
        //    path: path.clone(),
        //    line_number: r.line_number as usize,
        //    line: r.line.trim_end().to_string(),
        //    matches: r.matches.clone(),
        //})
        .collect()
}

pub struct ResultsList {
    pub results: Vec<Result>,
    pub state: ListState,
}

impl Default for ResultsList {
    fn default() -> Self {
        Self {
            results: Vec::new(),
            state: ListState::default(),
        }
    }
}

#[derive(Clone)]
pub struct PreviewState {
    pub scroll: (u16, u16),
    pub file_name: Option<String>,
    //pub file_type: Option<String>,
    pub highlighted_lines: Vec<Vec<(Style, String)>>,
}

impl Default for PreviewState {
    fn default() -> Self {
        Self {
            scroll: (0, 0),
            file_name: None,
            //file_type: None,
            highlighted_lines: Vec::new(),
        }
    }
}

pub struct App {
    pub target_path: PathBuf,
    pub input: Input,
    pub pattern: String,
    pub current_block: CurrentBlock,
    pub should_quit: bool,
    pub results_list: ResultsList,
    pub results_queue: Arc<SegQueue<Result>>,
    pub preview_state: PreviewState,
    pub preview_cache: HashMap<PathBuf, Vec<Vec<(Style, String)>>>,
    pub preview_pane_height: u16,
    pub selected_result: Option<Result>,
    pub syntect_preview_theme: Theme,
    pub ratatui_theme_settings: RThemeSettings,
    pub syntax_set: SyntaxSet,
    pub search_thread_handle: Option<std::thread::JoinHandle<()>>,
}

impl Default for App {
    fn default() -> Self {
        let theme = Theme::default();
        let rtheme_settings = RThemeSettings::from(theme.settings.clone());
        Self {
            target_path: PathBuf::from(DEFAULT_PATH),
            input: Input::new(String::new()),
            pattern: String::new(),
            current_block: CurrentBlock::Search,
            should_quit: false,
            results_list: ResultsList::default(),
            results_queue: Arc::new(SegQueue::new()),
            preview_state: PreviewState::default(),
            preview_cache: HashMap::new(),
            preview_pane_height: 0,
            selected_result: None,
            syntect_preview_theme: theme,
            ratatui_theme_settings: rtheme_settings,
            syntax_set: SyntaxSet::load_defaults_newlines(),
            search_thread_handle: None,
        }
    }
}

impl App {
    fn get_current_block_index(&self) -> usize {
        BLOCKS
            .iter()
            .position(|&block| block == self.current_block)
            .unwrap()
    }

    pub fn next_block(&mut self) {
        let current_index = self.get_current_block_index();
        let next_index = (current_index + 1) % BLOCKS.len();
        self.current_block = BLOCKS[next_index];
    }

    pub fn previous_block(&mut self) {
        let current_index = self.get_current_block_index();
        let previous_index = if current_index == 0 {
            BLOCKS.len() - 1
        } else {
            current_index - 1
        };
        self.current_block = BLOCKS[previous_index];
    }

    /// ┌───────────────────┐┌─────────────┐
    /// │ Results           ││ Preview     │
    /// │                   ││             │
    /// │                   ││             │
    /// │                   ││             │
    /// └───────────────────┘│             │
    /// ┌───────────────────┐│             │
    /// │ Search          x ││             │
    /// └───────────────────┘└─────────────┘
    pub fn move_to_block_on_top(&mut self) {
        match self.current_block {
            CurrentBlock::Search => {
                self.current_block = CurrentBlock::Results;
            }
            _ => {}
        }
    }

    /// ┌───────────────────┐┌─────────────┐
    /// │ Results         x ││ Preview     │
    /// │                   ││             │
    /// │                   ││             │
    /// │                   ││             │
    /// └───────────────────┘│             │
    /// ┌───────────────────┐│             │
    /// │ Search            ││             │
    /// └───────────────────┘└─────────────┘
    pub fn move_to_block_below(&mut self) {
        match self.current_block {
            CurrentBlock::Results => {
                self.current_block = CurrentBlock::Search;
            }
            _ => {}
        }
    }

    /// ┌───────────────────┐┌─────────────┐
    /// │ Results         x ││ Preview     │
    /// │                   ││             │
    /// │                   ││             │
    /// │                   ││             │
    /// └───────────────────┘│             │
    /// ┌───────────────────┐│             │
    /// │ Search          x ││             │
    /// └───────────────────┘└─────────────┘
    pub fn move_to_block_right(&mut self) {
        match self.current_block {
            CurrentBlock::Results | CurrentBlock::Search => {
                self.current_block = CurrentBlock::Preview;
            }
            _ => {}
        }
    }

    /// ┌───────────────────┐┌─────────────┐
    /// │ Results           ││ Preview   x │
    /// │                   ││             │
    /// │                   ││             │
    /// │                   ││             │
    /// └───────────────────┘│             │
    /// ┌───────────────────┐│             │
    /// │ Search            ││             │
    /// └───────────────────┘└─────────────┘
    pub fn move_to_block_left(&mut self) {
        match self.current_block {
            CurrentBlock::Preview => {
                self.current_block = CurrentBlock::Results;
            }
            _ => {}
        }
    }

    pub fn select_result(&mut self, index: usize) {
        self.selected_result = Some(self.results_list.results[index].clone());
    }

    pub fn compute_highlights(&mut self, file_path: &Path) {
        if self.preview_cache.get(file_path).is_none() {
            debug!("Computing highlights for {:?}", file_path);
            let mut highlighter =
                HighlightFile::new(file_path, &self.syntax_set, &self.syntect_preview_theme)
                    .unwrap();
            let mut line = String::new();
            let mut highlighted_lines = Vec::new();
            while highlighter.reader.read_line(&mut line).unwrap() > 0 {
                {
                    let line_regions = highlighter
                        .highlight_lines
                        .highlight_line(&line, &self.syntax_set)
                        .unwrap();

                    let mut cloned_regions = Vec::new();
                    for region in line_regions.iter().take(line.len() - 1) {
                        cloned_regions.push((region.0, region.1.to_owned()));
                    }
                    let last_region = line_regions.last().unwrap();
                    cloned_regions.push((last_region.0, last_region.1.trim_end().to_owned()));

                    highlighted_lines.push(cloned_regions);
                }
                line.clear();
            }
            self.preview_cache
                .insert(file_path.to_path_buf(), highlighted_lines.clone());
        }
        self.preview_state.highlighted_lines = self.preview_cache[file_path].clone();
    }

    pub fn set_scroll_for_result(&mut self, result: &Result) {
        self.preview_state.scroll = (
            (result.line_number as isize - (self.preview_pane_height as isize) / 3 + 1).max(0)
                as u16,
            0,
        );
    }

    pub fn scroll_preview_down(&mut self, offset: u16) {
        self.preview_state.scroll.0 = (self.preview_state.scroll.0 + offset).min(
            (self.preview_state.highlighted_lines.len() as isize
                - (2 * self.preview_pane_height / 3) as isize)
                .max(0) as u16,
        );
    }

    pub fn scroll_preview_up(&mut self, offset: u16) {
        self.preview_state.scroll.0 = self.preview_state.scroll.0.saturating_sub(offset);
    }

    pub fn reset_preview_scroll(&mut self) {
        self.preview_state.scroll = (0, 0);
    }

    pub fn preview_theme(mut self, theme: &Theme) -> Self {
        self.syntect_preview_theme = theme.clone();
        self.ratatui_theme_settings = RThemeSettings::from(theme.settings.clone());
        self
    }
}
