use crate::ui::input::Input;
use crate::utils::strings::EMPTY_STRING;
use ratatui::widgets::ListState;

#[derive(Debug)]
pub struct Picker {
    pub(crate) state: ListState,
    pub(crate) relative_state: ListState,
    pub(crate) view_offset: usize,
    _inverted: bool,
    pub(crate) input: Input,
}

impl Default for Picker {
    fn default() -> Self {
        Self::new()
    }
}

impl Picker {
    fn new() -> Self {
        Self {
            state: ListState::default(),
            relative_state: ListState::default(),
            view_offset: 0,
            _inverted: false,
            input: Input::new(EMPTY_STRING.to_string()),
        }
    }

    pub(crate) fn inverted(mut self) -> Self {
        self._inverted = !self._inverted;
        self
    }

    pub(crate) fn reset_selection(&mut self) {
        self.state.select(Some(0));
        self.relative_state.select(Some(0));
        self.view_offset = 0;
    }

    pub(crate) fn reset_input(&mut self) {
        self.input.reset();
    }

    pub(crate) fn selected(&self) -> Option<usize> {
        self.state.selected()
    }

    pub(crate) fn select(&mut self, index: Option<usize>) {
        self.state.select(index);
    }

    fn relative_selected(&self) -> Option<usize> {
        self.relative_state.selected()
    }

    pub(crate) fn relative_select(&mut self, index: Option<usize>) {
        self.relative_state.select(index);
    }

    pub(crate) fn select_next(&mut self, total_items: usize, height: usize) {
        if self._inverted {
            self._select_prev(total_items, height);
        } else {
            self._select_next(total_items, height);
        }
    }

    pub(crate) fn select_prev(&mut self, total_items: usize, height: usize) {
        if self._inverted {
            self._select_next(total_items, height);
        } else {
            self._select_prev(total_items, height);
        }
    }

    fn _select_next(&mut self, total_items: usize, height: usize) {
        let selected = self.selected().unwrap_or(0);
        let relative_selected = self.relative_selected().unwrap_or(0);
        if selected > 0 {
            self.select(Some(selected - 1));
            self.relative_select(Some(relative_selected.saturating_sub(1)));
            if relative_selected == 0 {
                self.view_offset = self.view_offset.saturating_sub(1);
            }
        } else {
            self.view_offset = total_items.saturating_sub(height - 2);
            self.select(Some(total_items.saturating_sub(1)));
            self.relative_select(Some(height - 3));
        }
    }

    fn _select_prev(&mut self, total_items: usize, height: usize) {
        let new_index = (self.selected().unwrap_or(0) + 1) % total_items;
        self.select(Some(new_index));
        if new_index == 0 {
            self.view_offset = 0;
            self.relative_select(Some(0));
            return;
        }
        if self.relative_selected().unwrap_or(0) == height - 3 {
            self.view_offset += 1;
            self.relative_select(Some(
                self.selected().unwrap_or(0).min(height - 3),
            ));
        } else {
            self.relative_select(Some(
                (self.relative_selected().unwrap_or(0) + 1)
                    .min(self.selected().unwrap_or(0)),
            ));
        }
    }
}
