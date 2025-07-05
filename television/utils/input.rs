// Adapted from https://github.com/sayanarijit/tui-input
//
// MIT License

// Copyright (c) 2021 Arijit Basu
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

/// Input requests are used to change the input state.
///
/// Different backends can be used to convert events into requests.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, PartialOrd, PartialEq, Eq, Clone, Copy, Hash)]
pub enum InputRequest {
    SetCursor(usize),
    InsertChar(char),
    GoToPrevChar,
    GoToNextChar,
    GoToPrevWord,
    GoToNextWord,
    GoToStart,
    GoToEnd,
    DeletePrevChar,
    DeleteNextChar,
    DeletePrevWord,
    DeleteNextWord,
    DeleteLine,
    DeleteTillEnd,
}

#[derive(Debug, PartialOrd, PartialEq, Eq, Clone, Copy, Hash)]
pub struct StateChanged {
    pub value: bool,
    pub cursor: bool,
}

#[allow(clippy::module_name_repetitions)]
pub type InputResponse = Option<StateChanged>;

/// An input buffer with cursor support.
#[derive(Default, Debug, Clone, PartialEq, Hash)]
pub struct Input {
    value: String,
    cursor: usize,
}

impl Input {
    /// Initialize a new instance with a given value
    /// Cursor will be set to the given value's length.
    pub fn new(value: String) -> Self {
        let len = value.chars().count();
        Self { value, cursor: len }
    }

    /// Set the value manually.
    /// Cursor will be set to the given value's length.
    pub fn with_value(mut self, value: String) -> Self {
        self.cursor = value.chars().count();
        self.value = value;
        self
    }

    /// Set the cursor manually.
    /// If the input is larger than the value length, it'll be auto adjusted.
    pub fn with_cursor(mut self, cursor: usize) -> Self {
        self.cursor = cursor.min(self.value.chars().count());
        self
    }

    // Reset the cursor and value to default
    pub fn reset(&mut self) {
        self.cursor = Default::default();
        self.value = String::default();
    }

    /// Handle request and emit response.
    #[allow(clippy::too_many_lines)]
    pub fn handle(&mut self, req: InputRequest) -> InputResponse {
        use InputRequest::{
            DeleteLine, DeleteNextChar, DeleteNextWord, DeletePrevChar,
            DeletePrevWord, DeleteTillEnd, GoToEnd, GoToNextChar,
            GoToNextWord, GoToPrevChar, GoToPrevWord, GoToStart, InsertChar,
            SetCursor,
        };
        match req {
            SetCursor(pos) => {
                let pos = pos.min(self.value.chars().count());
                if self.cursor == pos {
                    None
                } else {
                    self.cursor = pos;
                    Some(StateChanged {
                        value: false,
                        cursor: true,
                    })
                }
            }
            InsertChar(c) => {
                if self.cursor == self.value.chars().count() {
                    self.value.push(c);
                } else {
                    self.value = self
                        .value
                        .chars()
                        .take(self.cursor)
                        .chain(
                            std::iter::once(c)
                                .chain(self.value.chars().skip(self.cursor)),
                        )
                        .collect();
                }
                self.cursor += 1;
                Some(StateChanged {
                    value: true,
                    cursor: true,
                })
            }

            DeletePrevChar => {
                if self.cursor == 0 {
                    None
                } else {
                    self.cursor -= 1;
                    self.value = self
                        .value
                        .chars()
                        .enumerate()
                        .filter(|(i, _)| i != &self.cursor)
                        .map(|(_, c)| c)
                        .collect();

                    Some(StateChanged {
                        value: true,
                        cursor: true,
                    })
                }
            }

            DeleteNextChar => {
                if self.cursor == self.value.chars().count() {
                    None
                } else {
                    self.value = self
                        .value
                        .chars()
                        .enumerate()
                        .filter(|(i, _)| i != &self.cursor)
                        .map(|(_, c)| c)
                        .collect();
                    Some(StateChanged {
                        value: true,
                        cursor: false,
                    })
                }
            }

            GoToPrevChar => {
                if self.cursor == 0 {
                    None
                } else {
                    self.cursor -= 1;
                    Some(StateChanged {
                        value: false,
                        cursor: true,
                    })
                }
            }

            GoToPrevWord => {
                if self.cursor == 0 {
                    None
                } else {
                    self.cursor = self
                        .value
                        .chars()
                        .rev()
                        .skip(
                            self.value.chars().count().max(self.cursor)
                                - self.cursor,
                        )
                        .skip_while(|c| !c.is_alphanumeric())
                        .skip_while(|c| c.is_alphanumeric())
                        .count();
                    Some(StateChanged {
                        value: false,
                        cursor: true,
                    })
                }
            }

            GoToNextChar => {
                if self.cursor == self.value.chars().count() {
                    None
                } else {
                    self.cursor += 1;
                    Some(StateChanged {
                        value: false,
                        cursor: true,
                    })
                }
            }

            GoToNextWord => {
                if self.cursor == self.value.chars().count() {
                    None
                } else {
                    self.cursor = self
                        .value
                        .chars()
                        .enumerate()
                        .skip(self.cursor)
                        .skip_while(|(_, c)| c.is_alphanumeric())
                        .find(|(_, c)| c.is_alphanumeric())
                        .map(|(i, _)| i)
                        .unwrap_or_else(|| self.value.chars().count());

                    Some(StateChanged {
                        value: false,
                        cursor: true,
                    })
                }
            }

            DeleteLine => {
                if self.value.is_empty() {
                    None
                } else {
                    let cursor = self.cursor;
                    self.value = String::new();
                    self.cursor = 0;
                    Some(StateChanged {
                        value: true,
                        cursor: self.cursor == cursor,
                    })
                }
            }

            DeletePrevWord => {
                if self.cursor == 0 {
                    None
                } else {
                    let remaining = self.value.chars().skip(self.cursor);
                    let rev = self
                        .value
                        .chars()
                        .rev()
                        .skip(
                            self.value.chars().count().max(self.cursor)
                                - self.cursor,
                        )
                        .skip_while(|c| !c.is_alphanumeric())
                        .skip_while(|c| c.is_alphanumeric())
                        .collect::<Vec<char>>();
                    let rev_len = rev.len();
                    self.value =
                        rev.into_iter().rev().chain(remaining).collect();
                    self.cursor = rev_len;
                    Some(StateChanged {
                        value: true,
                        cursor: true,
                    })
                }
            }

            DeleteNextWord => {
                if self.cursor == self.value.chars().count() {
                    None
                } else {
                    self.value = self
                        .value
                        .chars()
                        .take(self.cursor)
                        .chain(
                            self.value
                                .chars()
                                .skip(self.cursor)
                                .skip_while(|c| c.is_alphanumeric())
                                .skip_while(|c| !c.is_alphanumeric()),
                        )
                        .collect();

                    Some(StateChanged {
                        value: true,
                        cursor: false,
                    })
                }
            }

            GoToStart => {
                if self.cursor == 0 {
                    None
                } else {
                    self.cursor = 0;
                    Some(StateChanged {
                        value: false,
                        cursor: true,
                    })
                }
            }

            GoToEnd => {
                let count = self.value.chars().count();
                if self.cursor == count {
                    None
                } else {
                    self.cursor = count;
                    Some(StateChanged {
                        value: false,
                        cursor: true,
                    })
                }
            }

            DeleteTillEnd => {
                self.value = self.value.chars().take(self.cursor).collect();
                Some(StateChanged {
                    value: true,
                    cursor: false,
                })
            }
        }
    }

    /// Get a reference to the current value.
    pub fn value(&self) -> &str {
        self.value.as_str()
    }

    /// Get the correct cursor placement.
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Get the current cursor position with account for multi space characters.
    pub fn visual_cursor(&self) -> usize {
        if self.cursor == 0 {
            return 0;
        }

        // Safe, because the end index will always be within bounds
        unicode_width::UnicodeWidthStr::width(unsafe {
            self.value.get_unchecked(
                0..self
                    .value
                    .char_indices()
                    .nth(self.cursor)
                    .map_or_else(|| self.value.len(), |(index, _)| index),
            )
        })
    }

    /// Get the scroll position with account for multi space characters.
    pub fn visual_scroll(&self, width: usize) -> usize {
        let scroll = self.visual_cursor().max(width) - width;
        let mut uscroll = 0;
        let mut chars = self.value().chars();

        while uscroll < scroll {
            match chars.next() {
                Some(c) => {
                    uscroll +=
                        unicode_width::UnicodeWidthChar::width(c).unwrap_or(0);
                }
                None => break,
            }
        }
        uscroll
    }
}

impl From<Input> for String {
    fn from(input: Input) -> Self {
        input.value
    }
}

impl From<String> for Input {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for Input {
    fn from(value: &str) -> Self {
        Self::new(value.into())
    }
}

impl std::fmt::Display for Input {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    const TEXT: &str = "first second, third.";

    use super::*;

    #[test]
    fn set_cursor() {
        let mut input: Input = TEXT.into();

        let req = InputRequest::SetCursor(3);
        let resp = input.handle(req);

        assert_eq!(
            resp,
            Some(StateChanged {
                value: false,
                cursor: true,
            })
        );

        assert_eq!(input.value(), "first second, third.");
        assert_eq!(input.cursor(), 3);

        let req = InputRequest::SetCursor(30);
        let resp = input.handle(req);

        assert_eq!(input.cursor(), TEXT.chars().count());
        assert_eq!(
            resp,
            Some(StateChanged {
                value: false,
                cursor: true,
            })
        );

        let req = InputRequest::SetCursor(TEXT.chars().count());
        let resp = input.handle(req);

        assert_eq!(input.cursor(), TEXT.chars().count());
        assert_eq!(resp, None);
    }

    #[test]
    fn insert_char() {
        let mut input: Input = TEXT.into();

        let req = InputRequest::InsertChar('x');
        let resp = input.handle(req);

        assert_eq!(
            resp,
            Some(StateChanged {
                value: true,
                cursor: true,
            })
        );

        assert_eq!(input.value(), "first second, third.x");
        assert_eq!(input.cursor(), TEXT.chars().count() + 1);
        input.handle(req);
        assert_eq!(input.value(), "first second, third.xx");
        assert_eq!(input.cursor(), TEXT.chars().count() + 2);

        let mut input = input.with_cursor(3);
        input.handle(req);
        assert_eq!(input.value(), "firxst second, third.xx");
        assert_eq!(input.cursor(), 4);

        input.handle(req);
        assert_eq!(input.value(), "firxxst second, third.xx");
        assert_eq!(input.cursor(), 5);
    }

    #[test]
    fn go_to_prev_char() {
        let mut input: Input = TEXT.into();

        let req = InputRequest::GoToPrevChar;
        let resp = input.handle(req);

        assert_eq!(
            resp,
            Some(StateChanged {
                value: false,
                cursor: true,
            })
        );

        assert_eq!(input.value(), "first second, third.");
        assert_eq!(input.cursor(), TEXT.chars().count() - 1);

        let mut input = input.with_cursor(3);
        input.handle(req);
        assert_eq!(input.value(), "first second, third.");
        assert_eq!(input.cursor(), 2);

        input.handle(req);
        assert_eq!(input.value(), "first second, third.");
        assert_eq!(input.cursor(), 1);
    }

    #[test]
    fn remove_unicode_chars() {
        let mut input: Input = "¡test¡".into();

        let req = InputRequest::DeletePrevChar;
        let resp = input.handle(req);

        assert_eq!(
            resp,
            Some(StateChanged {
                value: true,
                cursor: true,
            })
        );

        assert_eq!(input.value(), "¡test");
        assert_eq!(input.cursor(), 5);

        input.handle(InputRequest::GoToStart);

        let req = InputRequest::DeleteNextChar;
        let resp = input.handle(req);

        assert_eq!(
            resp,
            Some(StateChanged {
                value: true,
                cursor: false,
            })
        );

        assert_eq!(input.value(), "test");
        assert_eq!(input.cursor(), 0);
    }

    #[test]
    fn insert_unicode_chars() {
        let mut input = Input::from("¡test¡").with_cursor(5);

        let req = InputRequest::InsertChar('☆');
        let resp = input.handle(req);

        assert_eq!(
            resp,
            Some(StateChanged {
                value: true,
                cursor: true,
            })
        );

        assert_eq!(input.value(), "¡test☆¡");
        assert_eq!(input.cursor(), 6);

        input.handle(InputRequest::GoToStart);
        input.handle(InputRequest::GoToNextChar);

        let req = InputRequest::InsertChar('☆');
        let resp = input.handle(req);

        assert_eq!(
            resp,
            Some(StateChanged {
                value: true,
                cursor: true,
            })
        );

        assert_eq!(input.value(), "¡☆test☆¡");
        assert_eq!(input.cursor(), 2);
    }

    #[test]
    fn multispace_characters() {
        let input: Input = "Ｈｅｌｌｏ, ｗｏｒｌｄ!".into();
        assert_eq!(input.cursor(), 13);
        assert_eq!(input.visual_cursor(), 23);
        assert_eq!(input.visual_scroll(6), 18);
    }
}
