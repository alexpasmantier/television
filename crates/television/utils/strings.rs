use lazy_static::lazy_static;
use std::fmt::Write;

pub fn next_char_boundary(s: &str, start: usize) -> usize {
    let mut i = start;
    while !s.is_char_boundary(i) {
        i += 1;
    }
    i
}

pub fn prev_char_boundary(s: &str, start: usize) -> usize {
    let mut i = start;
    while !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}

pub fn slice_at_char_boundaries(
    s: &str,
    start_byte_index: usize,
    end_byte_index: usize,
) -> &str {
    &s[prev_char_boundary(s, start_byte_index)
        ..next_char_boundary(s, end_byte_index)]
}

pub fn slice_up_to_char_boundary(s: &str, byte_index: usize) -> &str {
    let mut char_index = byte_index;
    while !s.is_char_boundary(char_index) {
        char_index -= 1;
    }
    &s[..char_index]
}

fn try_parse_utf8_char(input: &[u8]) -> Option<(char, usize)> {
    let str_from_utf8 = |seq| std::str::from_utf8(seq).ok();

    let decoded = input
        .get(0..1)
        .and_then(str_from_utf8)
        .map(|c| (c, 1))
        .or_else(|| input.get(0..2).and_then(str_from_utf8).map(|c| (c, 2)))
        .or_else(|| input.get(0..3).and_then(str_from_utf8).map(|c| (c, 3)))
        .or_else(|| input.get(0..4).and_then(str_from_utf8).map(|c| (c, 4)));

    decoded.map(|(seq, n)| (seq.chars().next().unwrap(), n))
}

lazy_static! {
    static ref NULL_SYMBOL: char = char::from_u32(0x2400).unwrap();
}

pub const EMPTY_STRING: &str = "";
pub const FOUR_SPACES: &str = "    ";
pub const TAB_WIDTH: usize = 4;

const SPACE_CHARACTER: char = ' ';
const TAB_CHARACTER: char = '\t';
const LINE_FEED_CHARACTER: char = '\x0A';
const DELETE_CHARACTER: char = '\x7F';
const BOM_CHARACTER: char = '\u{FEFF}';
const NULL_CHARACTER: char = '\x00';
const UNIT_SEPARATOR_CHARACTER: char = '\u{001F}';
const APPLICATION_PROGRAM_COMMAND_CHARACTER: char = '\u{009F}';

pub fn replace_nonprintable(input: &[u8], tab_width: usize) -> String {
    let mut output = String::new();

    let mut idx = 0;
    let len = input.len();
    while idx < len {
        if let Some((chr, skip_ahead)) = try_parse_utf8_char(&input[idx..]) {
            idx += skip_ahead;

            match chr {
                // space
                SPACE_CHARACTER => output.push(' '),
                // tab
                TAB_CHARACTER => output.push_str(&" ".repeat(tab_width)),
                // line feed
                LINE_FEED_CHARACTER => {
                    output.push_str("␊\x0A");
                }
                // ASCII control characters from 0x00 to 0x1F
                // + control characters from \u{007F} to \u{009F}
                NULL_CHARACTER..=UNIT_SEPARATOR_CHARACTER
                | DELETE_CHARACTER..=APPLICATION_PROGRAM_COMMAND_CHARACTER => {
                    output.push(*NULL_SYMBOL);
                }
                // don't print BOMs
                BOM_CHARACTER => {}
                // unicode characters above 0x0700 seem unstable with ratatui
                c if c > '\u{0700}' => {
                    output.push(*NULL_SYMBOL);
                }
                // everything else
                c => output.push(c),
            }
        } else {
            write!(output, "\\x{:02X}", input[idx]).ok();
            idx += 1;
        }
    }

    output
}

/// The threshold for considering a buffer to be printable ASCII.
///
/// This is used to determine whether a file is likely to be a text file
/// based on a sample of its contents.
pub const PRINTABLE_ASCII_THRESHOLD: f32 = 0.7;

pub fn proportion_of_printable_ascii_characters(buffer: &[u8]) -> f32 {
    let mut printable = 0;
    for &byte in buffer {
        if byte > 32 && byte < 127 {
            printable += 1;
        }
    }
    printable as f32 / buffer.len() as f32
}

const MAX_LINE_LENGTH: usize = 500;

pub fn preprocess_line(line: &str) -> String {
    replace_nonprintable(
        {
            if line.len() > MAX_LINE_LENGTH {
                slice_up_to_char_boundary(line, MAX_LINE_LENGTH)
            } else {
                line
            }
        }
        .trim_end_matches(['\r', '\n', '\0'])
        .as_bytes(),
        TAB_WIDTH,
    )
}

pub fn shrink_with_ellipsis(s: &str, max_length: usize) -> String {
    if s.len() <= max_length {
        return s.to_string();
    }

    let half_max_length = (max_length / 2).saturating_sub(2);
    let first_half = slice_up_to_char_boundary(s, half_max_length);
    let second_half =
        slice_at_char_boundaries(s, s.len() - half_max_length, s.len());
    format!("{first_half}…{second_half}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_replace_nonprintable(input: &str, expected: &str) {
        let actual = replace_nonprintable(input.as_bytes(), 2);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_replace_nonprintable_ascii() {
        test_replace_nonprintable("Hello, World!", "Hello, World!");
    }

    #[test]
    fn test_replace_nonprintable_tab() {
        test_replace_nonprintable("Hello\tWorld!", "Hello  World!");
    }

    #[test]
    fn test_replace_nonprintable_line_feed() {
        test_replace_nonprintable("Hello\nWorld!", "Hello␊\nWorld!");
    }

    #[test]
    fn test_replace_nonprintable_null() {
        test_replace_nonprintable("Hello\x00World!", "Hello␀World!");
        test_replace_nonprintable("Hello World!\0", "Hello World!␀");
    }

    #[test]
    fn test_replace_nonprintable_delete() {
        test_replace_nonprintable("Hello\x7FWorld!", "Hello␀World!");
    }

    #[test]
    fn test_replace_nonprintable_bom() {
        test_replace_nonprintable("Hello\u{FEFF}World!", "HelloWorld!");
    }

    #[test]
    fn test_replace_nonprintable_start_txt() {
        test_replace_nonprintable("Àì", "Àì␀");
    }
}
