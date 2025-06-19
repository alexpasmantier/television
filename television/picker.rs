use crate::utils::{input::Input, strings::EMPTY_STRING};
use ratatui::widgets::ListState;

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Picker<T> {
    pub(crate) state: ListState,
    pub(crate) relative_state: ListState,
    inverted: bool,
    pub(crate) input: Input,
    pub entries: Vec<T>,
    pub total_items: u32,
}

impl<T> Default for Picker<T> {
    fn default() -> Self {
        Self::new(None)
    }
}

impl<T> Picker<T> {
    pub fn new(input: Option<String>) -> Self {
        Self {
            state: ListState::default(),
            relative_state: ListState::default(),
            inverted: false,
            input: Input::new(input.unwrap_or(EMPTY_STRING.to_string())),
            entries: Vec::new(),
            total_items: 0,
        }
    }

    pub(crate) fn offset(&self) -> usize {
        self.selected()
            .unwrap_or(0)
            .saturating_sub(self.relative_selected().unwrap_or(0))
    }

    pub(crate) fn inverted(mut self) -> Self {
        self.inverted = !self.inverted;
        self
    }

    pub(crate) fn reset_selection(&mut self) {
        self.state.select(Some(0));
        self.relative_state.select(Some(0));
    }

    pub(crate) fn reset_input(&mut self) {
        self.input.reset();
    }

    pub(crate) fn select_next(
        &mut self,
        step: u32,
        total_items: usize,
        height: usize,
    ) {
        if self.inverted {
            for _ in 0..step {
                self.inner_prev(total_items, height);
            }
        } else {
            for _ in 0..step {
                self.inner_next(total_items, height);
            }
        }
    }

    pub(crate) fn select_prev(
        &mut self,
        step: u32,
        total_items: usize,
        height: usize,
    ) {
        if self.inverted {
            for _ in 0..step {
                self.inner_next(total_items, height);
            }
        } else {
            for _ in 0..step {
                self.inner_prev(total_items, height);
            }
        }
    }

    fn inner_next(&mut self, total_items: usize, height: usize) {
        let selected = self.selected().unwrap_or(0);
        let relative_selected = self.relative_selected().unwrap_or(0);

        let new_selected = selected.wrapping_add(1) % total_items;
        self.select(Some(new_selected));

        if new_selected == 0 {
            self.relative_select(Some(0));
        } else {
            self.relative_select(Some(
                (relative_selected + 1).min(height - 1),
            ));
        }
    }

    fn inner_prev(&mut self, total_items: usize, height: usize) {
        let selected = self.selected().unwrap_or(0);
        let relative_selected = self.relative_selected().unwrap_or(0);

        let new_selected =
            selected.wrapping_add(total_items - 1) % total_items;
        self.select(Some(new_selected));

        if new_selected == total_items - 1 {
            self.relative_select(Some((height - 1).min(total_items - 1)));
        } else {
            self.relative_select(Some(relative_selected.saturating_sub(1)));
        }
    }
    /// Generic cursor movement helper.
    pub fn move_cursor(
        &mut self,
        movement: Movement,
        step: u32,
        total_items: usize,
        viewport_height: usize,
    ) {
        // Early return for empty collections
        if total_items == 0 {
            return;
        }

        match movement {
            Movement::Next => {
                self.select_next(step, total_items, viewport_height);
            }
            Movement::Prev => {
                self.select_prev(step, total_items, viewport_height);
            }
        }
    }
}

impl<Entry> Picker<Entry> {
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Movement {
    Next,
    Prev,
}

#[allow(clippy::doc_overindented_list_items)]
#[cfg(test)]
mod tests {
    use crate::channels::entry::Entry;

    use super::*;

    /// - item 0 S     R *
    /// - item 1 next    *
    /// - item 2         * height
    /// - item 3
    #[test]
    fn test_picker_select_next_default() {
        let mut picker = Picker::<Entry>::default();
        picker.select(Some(0));
        picker.relative_select(Some(0));
        picker.select_next(1, 4, 3);
        assert_eq!(picker.selected(), Some(1), "selected");
        assert_eq!(picker.relative_selected(), Some(1), "relative_selected");
    }

    /// - item 0         *
    /// - item 1 S     R *
    /// - item 2 next    * height
    /// - item 3
    #[test]
    fn test_picker_select_next_before_relative_last() {
        let mut picker = Picker::<Entry>::default();
        picker.select(Some(1));
        picker.relative_select(Some(1));
        picker.select_next(1, 4, 3);
        assert_eq!(picker.selected(), Some(2), "selected");
        assert_eq!(picker.relative_selected(), Some(2), "relative_selected");
    }

    /// - item 0         *
    /// - item 1         *
    /// - item 2 S     R * height
    /// - item 3 next
    #[test]
    fn test_picker_select_next_relative_last() {
        let mut picker = Picker::<Entry>::default();
        picker.select(Some(2));
        picker.relative_select(Some(2));
        picker.select_next(1, 4, 3);
        assert_eq!(picker.selected(), Some(3), "selected");
        assert_eq!(picker.relative_selected(), Some(2), "relative_selected");
    }

    /// - item 0 next    *
    /// - item 1         *
    /// - item 2       R * height
    /// - item 3 S
    #[test]
    fn test_picker_select_next_last() {
        let mut picker = Picker::<Entry>::default();
        picker.select(Some(3));
        picker.relative_select(Some(2));
        picker.select_next(1, 4, 3);
        assert_eq!(picker.selected(), Some(0), "selected");
        assert_eq!(picker.relative_selected(), Some(0), "relative_selected");
    }

    /// - item 0 next   *
    /// - item 1        *
    /// - item 2 S    R *
    ///                 * height
    #[test]
    fn test_picker_select_next_less_items_than_height_last() {
        let mut picker = Picker::<Entry>::default();
        picker.select(Some(2));
        picker.relative_select(Some(2));
        picker.select_next(1, 3, 4);
        assert_eq!(picker.selected(), Some(0), "selected");
        assert_eq!(picker.relative_selected(), Some(0), "relative_selected");
    }

    /// - item 0 prev    *
    /// - item 1 S     R *
    /// - item 2         * height
    /// - item 3
    #[test]
    fn test_picker_select_prev_default() {
        let mut picker = Picker::<Entry>::default();
        picker.select(Some(1));
        picker.relative_select(Some(1));
        picker.select_prev(1, 4, 3);
        assert_eq!(picker.selected(), Some(0), "selected");
        assert_eq!(picker.relative_selected(), Some(0), "relative_selected");
    }

    /// - item 0 S     R *
    /// - item 1         *        *
    /// - item 2         * height *
    /// - item 3 prev             *
    #[test]
    fn test_picker_select_prev_first() {
        let mut picker = Picker::<Entry>::default();
        picker.select(Some(0));
        picker.relative_select(Some(0));
        picker.select_prev(1, 4, 3);
        assert_eq!(picker.selected(), Some(3), "selected");
        assert_eq!(picker.relative_selected(), Some(2), "relative_selected");
    }

    /// - item 0         *
    /// - item 1         *
    /// - item 2 prev  R * height
    /// - item 3 S
    #[test]
    fn test_picker_select_prev_relative_trailing() {
        let mut picker = Picker::<Entry>::default();
        picker.select(Some(3));
        picker.relative_select(Some(2));
        picker.select_prev(1, 4, 3);
        assert_eq!(picker.selected(), Some(2), "selected");
        assert_eq!(picker.relative_selected(), Some(1), "relative_selected");
    }

    /// - item 0         *
    /// - item 1 prev    *
    /// - item 2 S     R * height
    /// - item 3
    #[test]
    fn test_picker_select_prev_relative_sync() {
        let mut picker = Picker::<Entry>::default();
        picker.select(Some(2));
        picker.relative_select(Some(2));
        picker.select_prev(1, 4, 3);
        assert_eq!(picker.selected(), Some(1), "selected");
        assert_eq!(picker.relative_selected(), Some(1), "relative_selected");
    }

    #[test]
    fn test_picker_offset_default() {
        let picker = Picker::<Entry>::default();
        assert_eq!(picker.offset(), 0, "offset");
    }

    #[test]
    fn test_picker_offset_none() {
        let mut picker = Picker::<Entry>::default();
        picker.select(None);
        picker.relative_select(None);
        assert_eq!(picker.offset(), 0, "offset");
    }

    #[test]
    fn test_picker_offset() {
        let mut picker = Picker::<Entry>::default();
        picker.select(Some(1));
        picker.relative_select(Some(2));
        assert_eq!(picker.offset(), 0, "offset");
    }

    #[test]
    fn test_picker_inverted() {
        let mut picker = Picker::<Entry>::default();
        picker.select(Some(0));
        picker.relative_select(Some(0));
        picker.select_next(1, 4, 2);
        picker = picker.inverted();
        picker.select_next(1, 4, 2);
        assert!(picker.inverted, "inverted");
        assert_eq!(picker.selected(), Some(0), "selected");
        assert_eq!(picker.relative_selected(), Some(0), "relative_selected");
    }
}
