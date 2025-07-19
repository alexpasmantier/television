use lazy_regex::{Lazy, Regex, regex};

use crate::screen::result_item::ResultItem;

/// Returns the index of the next character boundary in the given string.
///
/// If the given index is already a character boundary, it is returned as is.
/// If the given index is out of bounds, the length of the string is returned.
///
/// # Examples
/// ```
/// use television::utils::strings::next_char_boundary;
///
/// let s = "Hello, World!";
/// assert_eq!(next_char_boundary(s, 0), 0);
/// assert_eq!(next_char_boundary(s, 1), 1);
/// assert_eq!(next_char_boundary(s, 13), 13);
/// assert_eq!(next_char_boundary(s, 30), 13);
///
/// let s = "👋🌍!";
/// assert_eq!(next_char_boundary(s, 0), 0);
/// assert_eq!(next_char_boundary(s, 1), 4);
/// assert_eq!(next_char_boundary(s, 4), 4);
/// assert_eq!(next_char_boundary(s, 7), 8);
/// assert_eq!(next_char_boundary(s, 8), 8);
/// ```
pub fn next_char_boundary(s: &str, start: usize) -> usize {
    let mut i = start;
    let len = s.len();
    if i >= len {
        return len;
    }
    while !s.is_char_boundary(i) && i < len {
        i += 1;
    }
    i
}

/// Returns the index of the previous character boundary in the given string.
///
/// If the given index is already a character boundary, it is returned as is.
/// If the given index is out of bounds, 0 is returned.
///
/// # Examples
/// ```
/// use television::utils::strings::prev_char_boundary;
///
/// let s = "Hello, World!";
/// assert_eq!(prev_char_boundary(s, 0), 0);
/// assert_eq!(prev_char_boundary(s, 1), 1);
/// assert_eq!(prev_char_boundary(s, 5), 5);
///
/// let s = "👋🌍!";
/// assert_eq!(prev_char_boundary(s, 0), 0);
/// assert_eq!(prev_char_boundary(s, 4), 4);
/// assert_eq!(prev_char_boundary(s, 6), 4);
/// ```
pub fn prev_char_boundary(s: &str, start: usize) -> usize {
    let mut i = start;
    while !s.is_char_boundary(i) && i > 0 {
        i -= 1;
    }
    i
}

/// Returns a slice of the given string that starts and ends at character boundaries.
///
/// If the given start index is greater than the end index, or if either index is out of bounds,
/// an empty string is returned.
///
/// # Examples
/// ```
/// use television::utils::strings::slice_at_char_boundaries;
///
/// let s = "Hello, World!";
/// assert_eq!(slice_at_char_boundaries(s, 0, 0), "");
/// assert_eq!(slice_at_char_boundaries(s, 0, 1), "H");
///
/// let s = "👋🌍!";
/// assert_eq!(slice_at_char_boundaries(s, 0, 0), "");
/// assert_eq!(slice_at_char_boundaries(s, 0, 2), "👋");
/// assert_eq!(slice_at_char_boundaries(s, 0, 5), "👋🌍");
/// ```
pub fn slice_at_char_boundaries(
    s: &str,
    start_byte_index: usize,
    end_byte_index: usize,
) -> &str {
    if start_byte_index > end_byte_index
        || start_byte_index > s.len()
        || end_byte_index > s.len()
    {
        return EMPTY_STRING;
    }
    &s[prev_char_boundary(s, start_byte_index)
        ..next_char_boundary(s, end_byte_index)]
}

/// Returns a slice of the given string that starts at the beginning and ends at a character
/// boundary.
///
/// If the given index is out of bounds, the whole string is returned.
/// If the given index is already a character boundary, the string up to that index is returned.
///
/// # Examples
/// ```
/// use television::utils::strings::slice_up_to_char_boundary;
///
/// let s = "Hello, World!";
/// assert_eq!(slice_up_to_char_boundary(s, 0), "");
/// assert_eq!(slice_up_to_char_boundary(s, 1), "H");
/// assert_eq!(slice_up_to_char_boundary(s, 13), "Hello, World!");
///
/// let s = "👋\n🌍!";
/// assert_eq!(slice_up_to_char_boundary(s, 0), "");
/// assert_eq!(slice_up_to_char_boundary(s, 1), "👋");
/// assert_eq!(slice_up_to_char_boundary(s, 4), "👋");
/// assert_eq!(slice_up_to_char_boundary(s, 7), "👋\n🌍");
/// ```
pub fn slice_up_to_char_boundary(s: &str, byte_index: usize) -> &str {
    &s[..next_char_boundary(s, byte_index)]
}

/// Attempts to parse a UTF-8 character from the given byte slice.
///
/// The function returns the parsed character and the number of bytes consumed.
///
/// # Examples
/// ```
/// use television::utils::strings::try_parse_utf8_char;
///
/// let input = b"Hello, World!";
/// let (chr, n) = try_parse_utf8_char(input).unwrap();
/// assert_eq!(chr, 'H');
/// assert_eq!(n, 1);
///
/// let input = b"\xF0\x9F\x91\x8B\xF0\x9F\x8C\x8D!";
/// let (chr, n) = try_parse_utf8_char(input).unwrap();
/// assert_eq!(chr, '👋');
/// assert_eq!(n, 4);
/// ```
pub fn try_parse_utf8_char(input: &[u8]) -> Option<(char, usize)> {
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

pub const EMPTY_STRING: &str = "";
pub const TAB_WIDTH: usize = 4;

/// The Unicode symbol to use for non-printable characters.
const NULL_SYMBOL: char = '\u{2400}';
const TAB_CHARACTER: char = '\t';
const LINE_FEED_CHARACTER: char = '\x0A';
const CARRIAGE_RETURN_CHARACTER: char = '\r';
const DELETE_CHARACTER: char = '\x7F';
const BOM_CHARACTER: char = '\u{FEFF}';
const NULL_CHARACTER: char = '\x00';
const UNIT_SEPARATOR_CHARACTER: char = '\u{001F}';
const APPLICATION_PROGRAM_COMMAND_CHARACTER: char = '\u{009F}';

const NF_RANGE_DEVICONS: std::ops::RangeInclusive<char> =
    '\u{e700}'..='\u{e8ef}';
const NF_RANGE_SETI: std::ops::RangeInclusive<char> = '\u{e5fa}'..='\u{e6b7}';
const NF_RANGE_FONT_AWESOME: std::ops::RangeInclusive<char> =
    '\u{ed00}'..='\u{f2ff}';
const NF_RANGE_FONT_AWESOME_EXT: std::ops::RangeInclusive<char> =
    '\u{e200}'..='\u{e2a9}';
const NF_RANGE_MATERIAL: std::ops::RangeInclusive<char> =
    '\u{f0001}'..='\u{f1af0}';
const NF_RANGE_WEATHER: std::ops::RangeInclusive<char> =
    '\u{e300}'..='\u{e3e3}';
const NF_RANGE_OCTICONS_1: std::ops::RangeInclusive<char> =
    '\u{f400}'..='\u{f533}';
const NF_RANGE_OCTICONS_2: std::ops::RangeInclusive<char> =
    '\u{2665}'..='\u{26a1}';
const NF_RANGE_POWERLINE_1: std::ops::RangeInclusive<char> =
    '\u{e0a0}'..='\u{e0a2}';
const NF_RANGE_POWERLINE_2: std::ops::RangeInclusive<char> =
    '\u{e0b0}'..='\u{e0b3}';

const ALL_NF_RANGES: [&std::ops::RangeInclusive<char>; 10] = [
    &NF_RANGE_DEVICONS,
    &NF_RANGE_SETI,
    &NF_RANGE_FONT_AWESOME,
    &NF_RANGE_FONT_AWESOME_EXT,
    &NF_RANGE_MATERIAL,
    &NF_RANGE_WEATHER,
    &NF_RANGE_OCTICONS_1,
    &NF_RANGE_OCTICONS_2,
    &NF_RANGE_POWERLINE_1,
    &NF_RANGE_POWERLINE_2,
];

const VARIOUS_UNIT_WIDTH_SYMBOLS_RANGE: std::ops::RangeInclusive<char> =
    '\u{2000}'..='\u{25FF}';

pub struct ReplaceNonPrintableConfig {
    pub replace_tab: bool,
    pub tab_width: usize,
    pub replace_line_feed: bool,
    pub replace_control_characters: bool,
}

impl ReplaceNonPrintableConfig {
    pub fn tab_width(&mut self, tab_width: usize) -> &mut Self {
        self.tab_width = tab_width;
        self
    }

    pub fn keep_line_feed(&mut self) -> &mut Self {
        self.replace_line_feed = false;
        self
    }

    pub fn keep_control_characters(&mut self) -> &mut Self {
        self.replace_control_characters = false;
        self
    }
}

impl Default for ReplaceNonPrintableConfig {
    fn default() -> Self {
        Self {
            replace_tab: true,
            tab_width: TAB_WIDTH,
            replace_line_feed: true,
            replace_control_characters: true,
        }
    }
}

fn is_emoji(ch: char) -> bool {
    [
        // emoticons
        '\u{1F600}'..='\u{1F64F}',
        // misc. symbols and pictograms
        '\u{1F300}'..='\u{1F5FF}',
        // transports / map
        '\u{1F680}'..='\u{1F6FF}',
        // additional symbols and pictograms
        '\u{1F900}'..='\u{1F9FF}',
        // flags
        '\u{1F1E6}'..='\u{1F1FF}',
    ]
    .iter()
    .any(|range| range.contains(&ch))
}

#[allow(clippy::missing_panics_doc)]
/// Replaces non-printable characters in the given byte slice with default printable characters.
///
/// The tab width is used to determine how many spaces to replace a tab character with.
/// The default printable character for non-printable characters is the Unicode symbol for NULL.
///
/// The function returns a tuple containing the processed string and a vector of offsets introduced
/// by the transformation.
///
/// # Examples
/// ```
/// use television::utils::strings::{replace_non_printable, ReplaceNonPrintableConfig};
///
/// let input = b"Hello, World!";
/// let (output, offsets) = replace_non_printable(input, &ReplaceNonPrintableConfig::default());
/// assert_eq!(output, "Hello, World!");
/// assert_eq!(offsets, vec![0,0,0,0,0,0,0,0,0,0,0,0,0]);
///
/// let input = b"Hello,\tWorld!";
/// let (output, offsets) = replace_non_printable(input, &ReplaceNonPrintableConfig::default().tab_width(4));
/// assert_eq!(output, "Hello,    World!");
/// assert_eq!(offsets, vec![0,0,0,0,0,0,0,3,3,3,3,3,3]);
///
/// let input = b"Hello,\nWorld!";
/// let (output, offsets) = replace_non_printable(input, &ReplaceNonPrintableConfig::default());
/// assert_eq!(output, "Hello,World!");
/// assert_eq!(offsets, vec![0,0,0,0,0,0,0,-1,-1,-1,-1,-1,-1]);
/// ```
pub fn replace_non_printable(
    input: &[u8],
    config: &ReplaceNonPrintableConfig,
) -> (String, Vec<i16>) {
    let mut output = String::with_capacity(input.len());
    let mut offsets = Vec::new();
    let mut cumulative_offset: i16 = 0;

    let mut idx = 0;
    let len = input.len();
    while idx < len {
        offsets.push(cumulative_offset);
        if let Some((chr, skip_ahead)) = try_parse_utf8_char(&input[idx..]) {
            idx += skip_ahead;
            match chr {
                // tab
                TAB_CHARACTER if config.replace_tab => {
                    output.push_str(&" ".repeat(config.tab_width));
                    cumulative_offset +=
                        i16::try_from(config.tab_width).unwrap() - 1;
                }
                // line feed
                LINE_FEED_CHARACTER | CARRIAGE_RETURN_CHARACTER
                    if config.replace_line_feed =>
                {
                    cumulative_offset -= 1;
                }

                // Carriage return
                '\r' if config.replace_line_feed => {
                    // Do not add to output, just adjust offset
                    cumulative_offset -= 1;
                }

                // ASCII control characters from 0x00 to 0x1F
                // + control characters from \u{007F} to \u{009F}
                // + BOM
                NULL_CHARACTER..=UNIT_SEPARATOR_CHARACTER
                | DELETE_CHARACTER..=APPLICATION_PROGRAM_COMMAND_CHARACTER
                | BOM_CHARACTER
                    if config.replace_control_characters =>
                {
                    output.push(NULL_SYMBOL);
                }
                // CJK Unified Ideographs
                // ex: 解
                c if ('\u{4E00}'..='\u{9FFF}').contains(&c) => {
                    output.push(c);
                }
                // Korean: Hangul syllables
                // ex: 가 or 한
                c if ('\u{AC00}'..='\u{D7AF}').contains(&c) => {
                    output.push(c);
                }
                // some emojis
                // ex: 😀
                c if is_emoji(c) => {
                    output.push(c);
                }
                // Japanese (contiguous ranges for katakana and hiragana)
                // ex: katakana -> ア and hiragana -> あ
                c if ('\u{3040}'..='\u{30FF}').contains(&c) => {
                    output.push(c);
                }
                // Thai
                // ex: ส or ดี
                c if ('\u{0E00}'..='\u{0E7F}').contains(&c) => output.push(c),
                // Devanagari (most common Indic script)
                c if ('\u{0900}'..='\u{097F}').contains(&c) => output.push(c),
                // Nerd fonts
                c if ALL_NF_RANGES.iter().any(|r| r.contains(&c)) => {
                    output.push(c);
                }
                // Other unit width symbols
                c if VARIOUS_UNIT_WIDTH_SYMBOLS_RANGE.contains(&c) => {
                    output.push(c);
                }
                // Unicode characters above 0x0700 seem unstable with ratatui
                c if c > '\u{0700}' => {
                    output.push(NULL_SYMBOL);
                }
                // everything else
                c => output.push(c),
            }
        } else {
            output.push(NULL_SYMBOL);
            idx += 1;
        }
    }

    (output, offsets)
}

/// The threshold for considering a buffer to be printable ASCII.
///
/// This is used to determine whether a file is likely to be a text file
/// based on a sample of its contents.
pub const PRINTABLE_ASCII_THRESHOLD: f32 = 0.7;

/// Returns the proportion of printable ASCII characters in the given buffer.
///
/// This really is a cheap way to determine if a buffer is likely to be a text file.
///
/// # Examples
/// ```
/// use television::utils::strings::proportion_of_printable_ascii_characters;
///
/// let buffer = b"Hello, World!";
/// let proportion = proportion_of_printable_ascii_characters(buffer);
/// assert_eq!(proportion, 1.0);
///
/// let buffer = b"Hello, World!\x00";
/// let proportion = proportion_of_printable_ascii_characters(buffer);
/// assert_eq!(proportion, 0.9285714);
///
/// let buffer = b"\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0A\x0B\x0C\x0D\x0E\x0F";
/// let proportion = proportion_of_printable_ascii_characters(buffer);
/// assert_eq!(proportion, 0.0);
/// ```
pub fn proportion_of_printable_ascii_characters(buffer: &[u8]) -> f32 {
    let mut printable: usize = 0;
    for &byte in buffer {
        if (32..127).contains(&byte) {
            printable += 1;
        }
    }
    printable as f32 / buffer.len() as f32
}

const MAX_LINE_LENGTH: usize = 300;

/// Preprocesses a line of text for display.
///
/// This function trims the line, replaces non-printable characters, and truncates the line if it
/// is too long.
///
/// # Examples
/// ```
/// use television::utils::strings::preprocess_line;
///
/// let line = "Hello, World!";
/// let (processed, offsets) = preprocess_line(line);
/// assert_eq!(processed, "Hello, World!");
/// assert_eq!(offsets, vec![0,0,0,0,0,0,0,0,0,0,0,0,0]);
///
/// let line = "\x00World\x7F!";
/// let (processed, offsets) = preprocess_line(line);
/// assert_eq!(processed, "␀World␀!");
/// assert_eq!(offsets, vec![0,0,0,0,0,0,0,0]);
///
/// let line = "a".repeat(400);
/// let (processed, offsets) = preprocess_line(&line);
/// assert_eq!(processed.len(), 300);
/// assert_eq!(offsets, vec![0; 300]);
/// ```
pub fn preprocess_line(line: &str) -> (String, Vec<i16>) {
    replace_non_printable(
        {
            if line.len() > MAX_LINE_LENGTH {
                slice_up_to_char_boundary(line, MAX_LINE_LENGTH)
            } else {
                line
            }
        }
        .as_bytes(),
        &ReplaceNonPrintableConfig::default(),
    )
}

/// Make a matched string printable while preserving match ranges in the process.
///
/// This function preprocesses the matched string and returns a printable version of it along with
/// the match ranges adjusted to the new string.
///
/// # Examples
/// ```ignore
/// use television::channels::entry::Entry;
/// use television::utils::strings::make_result_item_printable;
///
/// let entry = Entry::new("Hello, World!".to_string()).with_match_indices(&[0, 7]);
/// let (printable, match_indices) = make_result_item_printable(&entry);
/// assert_eq!(printable, "Hello, World!");
/// assert_eq!(match_indices, vec![(0, 1), (7, 8)]);
///
/// let entry = Entry::new("Hello,\tWorld!".to_string()).with_match_indices(&[0, 10]);
/// let (printable, match_indices) = make_result_item_printable(&entry);
/// assert_eq!(printable, "Hello,    World!");
/// assert_eq!(match_indices, vec![(0, 1), (10, 11)]);
///
/// let entry = Entry::new("Hello,\nWorld!".to_string()).with_match_indices(&[0, 6]);
/// let (printable, match_indices) = make_result_item_printable(&entry);
/// assert_eq!(printable, "Hello,World!");
/// assert_eq!(match_indices, vec![(0, 1), (6, 7)]);
///
/// let entry = Entry::new("Hello, World!".to_string());
/// let (printable, match_indices) = make_result_item_printable(&entry);
/// assert_eq!(printable, "Hello, World!");
/// assert_eq!(match_indices, vec![]);
///
/// let entry = Entry::new("build.rs".to_string()).with_match_indices(&[0, 7]);
/// let (printable, match_indices) = make_result_item_printable(&entry);
/// assert_eq!(printable, "build.rs");
/// assert_eq!(match_indices, vec![(0, 1), (7, 8)]);
///
/// let entry = Entry::new("a\tb".to_string()).with_match_indices(&[0, 5]);
/// let (printable, match_indices) = make_result_item_printable(&entry);
/// assert_eq!(printable, "a    b");
/// assert_eq!(match_indices, vec![(0, 1), (5, 6)]);
///
/// let entry = Entry::new("a\tbcd".repeat(65)).with_match_indices(&[0, 330]);
/// let (printable, match_indices) = make_result_item_printable(&entry);
/// assert_eq!(printable.len(), 480);
/// assert_eq!(match_indices, vec![(0, 1)]);
///
/// let entry = Entry::new("ジェ abc".to_string()).with_match_indices(&[0, 2]);
/// let (printable, match_indices) = make_result_item_printable(&entry);
/// assert_eq!(printable, "ジェ abc");
/// assert_eq!(match_indices, vec![(0, 1), (2, 3)]);
/// ```
///
/// # Panics
/// This will panic if the length of the printable string or the match indices don't fit into a
/// `u32`.
pub fn make_result_item_printable(
    result_item: &(impl ResultItem + ?Sized),
) -> (String, Vec<(u32, u32)>) {
    // PERF: fast path for ascii
    if result_item.display().is_ascii() {
        match result_item.match_ranges() {
            // If there are no match ranges, we can return the display string directly
            None => {
                return (result_item.display().to_string(), Vec::new());
            }
            // Otherwise, check if we can return the display string without further processing
            Some(ranges) => {
                if !result_item
                    .display()
                    .chars()
                    .any(|c| c == '\t' || c == '\n' || c.is_control())
                {
                    return (
                        result_item.display().to_string(),
                        ranges.to_vec(),
                    );
                }
            }
        }
    }

    // Full processing for non-ASCII strings or strings that need preprocessing
    let (printable, transformation_offsets) =
        preprocess_line(result_item.display());
    let mut match_indices = Vec::new();

    if let Some(ranges) = result_item.match_ranges() {
        // PERF: Pre-allocate with known capacity
        match_indices.reserve(ranges.len());

        for (start, end) in ranges.iter().take_while(|(start, _)| {
            *start < u32::try_from(transformation_offsets.len()).unwrap()
        }) {
            let new_start = i64::from(*start)
                + i64::from(transformation_offsets[*start as usize]);
            let new_end = i64::from(*end)
                + i64::from(
                    // Use the last offset if the end index is out of bounds
                    // (this will be the case when the match range includes the last character)
                    transformation_offsets[(*end as usize)
                        .min(transformation_offsets.len() - 1)],
                );
            match_indices.push((
                u32::try_from(new_start).unwrap(),
                u32::try_from(new_end).unwrap(),
            ));
        }
    }

    (printable, match_indices)
}

/// Shrink a string to a maximum length, adding an ellipsis in the middle.
///
/// If the string is shorter than the maximum length, it is returned as is.
/// If the string is longer than the maximum length, it is shortened and an ellipsis is added in
/// the middle.
///
/// # Examples
/// ```
/// use television::utils::strings::shrink_with_ellipsis;
///
/// let s = "Hello, World!";
/// assert_eq!(shrink_with_ellipsis(s, 13), "Hello, World!");
/// assert_eq!(shrink_with_ellipsis(s, 6), "H…!");
/// ```
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

pub static CMD_RE: &Lazy<Regex> = regex!(r"\{(\d+)\}");

/// Formats a prototype string with the given template and source strings.
///
/// # Example
/// ```
/// use television::utils::strings::format_string;
///
/// let template = "cat {} {1}";
/// let source = "foo:bar:baz";
/// let delimiter = ":";
///
/// let formatted = format_string(template, source, delimiter);
/// assert_eq!(formatted, "cat 'foo:bar:baz' 'bar'");
/// ```
pub fn format_string(template: &str, source: &str, delimiter: &str) -> String {
    let parts = source.split(delimiter).collect::<Vec<&str>>();

    let mut formatted_string =
        template.replace("{}", format!("'{}'", source).as_str());

    formatted_string = CMD_RE
        .replace_all(&formatted_string, |caps: &regex::Captures| {
            let index =
                // these unwraps are safe because of the regex pattern
                caps.get(1).unwrap().as_str().parse::<usize>().unwrap();
            format!("'{}'", parts.get(index).unwrap_or(&""))
        })
        .to_string();

    formatted_string
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channels::entry::Entry;

    fn test_next_char_boundary(input: &str, start: usize, expected: usize) {
        let actual = next_char_boundary(input, start);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_next_char_boundary_ascii() {
        test_next_char_boundary("Hello, World!", 0, 0);
        test_next_char_boundary("Hello, World!", 1, 1);
        test_next_char_boundary("Hello, World!", 13, 13);
        test_next_char_boundary("Hello, World!", 30, 13);
    }

    #[test]
    fn test_next_char_boundary_emoji() {
        test_next_char_boundary("👋🌍!", 0, 0);
        test_next_char_boundary("👋🌍!", 1, 4);
        test_next_char_boundary("👋🌍!", 4, 4);
        test_next_char_boundary("👋🌍!", 8, 8);
        test_next_char_boundary("👋🌍!", 7, 8);
    }

    fn test_previous_char_boundary(
        input: &str,
        start: usize,
        expected: usize,
    ) {
        let actual = prev_char_boundary(input, start);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_previous_char_boundary_ascii() {
        test_previous_char_boundary("Hello, World!", 0, 0);
        test_previous_char_boundary("Hello, World!", 1, 1);
        test_previous_char_boundary("Hello, World!", 5, 5);
    }

    #[test]
    fn test_previous_char_boundary_emoji() {
        test_previous_char_boundary("👋🌍!", 0, 0);
        test_previous_char_boundary("👋🌍!", 4, 4);
        test_previous_char_boundary("👋🌍!", 6, 4);
        test_previous_char_boundary("👋🌍!", 8, 8);
    }

    fn test_slice_at_char_boundaries(
        input: &str,
        start: usize,
        end: usize,
        expected: &str,
    ) {
        let actual = slice_at_char_boundaries(input, start, end);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_slice_at_char_boundaries_ascii() {
        test_slice_at_char_boundaries("Hello, World!", 0, 0, "");
        test_slice_at_char_boundaries("Hello, World!", 0, 1, "H");
        test_slice_at_char_boundaries("Hello, World!", 0, 13, "Hello, World!");
        test_slice_at_char_boundaries("Hello, World!", 0, 30, "");
    }

    #[test]
    fn test_slice_at_char_boundaries_emoji() {
        test_slice_at_char_boundaries("👋🌍!", 0, 0, "");
        test_slice_at_char_boundaries("👋🌍!", 0, 4, "👋");
        test_slice_at_char_boundaries("👋🌍!", 0, 8, "👋🌍");
        test_slice_at_char_boundaries("👋🌍!", 0, 7, "👋🌍");
        test_slice_at_char_boundaries("👋🌍!", 0, 9, "👋🌍!");
    }

    fn test_replace_non_printable(input: &str, expected: &str) {
        let (actual, _offset) = replace_non_printable(
            input.as_bytes(),
            ReplaceNonPrintableConfig::default().tab_width(2),
        );
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_replace_non_printable_ascii() {
        test_replace_non_printable("Hello, World!", "Hello, World!");
    }

    #[test]
    fn test_replace_non_printable_tab() {
        test_replace_non_printable("Hello\tWorld!", "Hello  World!");
        test_replace_non_printable(
            "	-- AND
", "  -- AND",
        );
    }

    #[test]
    fn test_replace_non_printable_line_feed() {
        test_replace_non_printable("Hello\nWorld!", "HelloWorld!");
    }

    #[test]
    fn test_replace_non_printable_null() {
        test_replace_non_printable("Hello\x00World!", "Hello␀World!");
        test_replace_non_printable("Hello World!\0", "Hello World!␀");
    }

    #[test]
    fn test_replace_non_printable_delete() {
        test_replace_non_printable("Hello\x7FWorld!", "Hello␀World!");
    }

    #[test]
    fn test_replace_non_printable_bom() {
        test_replace_non_printable("Hello\u{FEFF}World!", "Hello␀World!");
    }

    #[test]
    fn test_replace_non_printable_start_txt() {
        test_replace_non_printable("Àì", "Àì␀");
    }

    #[test]
    fn test_replace_non_printable_range_tab() {
        let input = b"Hello,\tWorld!";
        let (output, offsets) = replace_non_printable(
            input,
            &ReplaceNonPrintableConfig::default(),
        );
        assert_eq!(output, "Hello,    World!");
        assert_eq!(offsets, vec![0, 0, 0, 0, 0, 0, 0, 3, 3, 3, 3, 3, 3]);
    }

    #[test]
    fn test_replace_non_printable_range_line_feed() {
        let input = b"Hello,\nWorld!";
        let (output, offsets) = replace_non_printable(
            input,
            ReplaceNonPrintableConfig::default().tab_width(2),
        );
        assert_eq!(output, "Hello,World!");
        assert_eq!(offsets, vec![0, 0, 0, 0, 0, 0, 0, -1, -1, -1, -1, -1, -1]);
    }

    #[test]
    fn test_cjk_characters() {
        let input = "你好,世界!".as_bytes();
        let config = ReplaceNonPrintableConfig::default();
        let (output, offsets) = replace_non_printable(input, &config);
        assert_eq!(output, "你好,世界!");
        assert_eq!(offsets, vec![0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_thai_characters() {
        let input = "สวัสดี!".as_bytes(); // สวัสดี is 6 characters + !
        let config = ReplaceNonPrintableConfig::default();
        let (output, offsets) = replace_non_printable(input, &config);
        assert_eq!(output, "สวัสดี!");
        assert_eq!(offsets, vec![0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_emoji_characters() {
        let input = "Hello 🌍!".as_bytes();
        let config = ReplaceNonPrintableConfig::default();
        let (output, offsets) = replace_non_printable(input, &config);
        assert_eq!(output, "Hello 🌍!");
        assert_eq!(offsets, vec![0, 0, 0, 0, 0, 0, 0, 0]);
    }
    #[test]
    fn test_devanagari_characters() {
        let input = "नमस्ते".as_bytes(); // नमस्ते is 6 characters
        let config = ReplaceNonPrintableConfig::default();
        let (output, offsets) = replace_non_printable(input, &config);
        assert_eq!(output, "नमस्ते");
        assert_eq!(offsets, vec![0, 0, 0, 0, 0, 0]);
    }
    #[test]
    fn test_hiragana_characters() {
        let input = "こんにちは".as_bytes();
        let config = ReplaceNonPrintableConfig::default();
        let (output, offsets) = replace_non_printable(input, &config);
        assert_eq!(output, "こんにちは");
        assert_eq!(offsets, vec![0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_katakana_characters() {
        let input = "コンニチハ".as_bytes();
        let config = ReplaceNonPrintableConfig::default();
        let (output, offsets) = replace_non_printable(input, &config);
        assert_eq!(output, "コンニチハ");
        assert_eq!(offsets, vec![0, 0, 0, 0, 0]);
    }
    #[test]
    fn test_korean_characters() {
        let input = "안녕하세요!".as_bytes();
        let config = ReplaceNonPrintableConfig::default();
        let (output, offsets) = replace_non_printable(input, &config);
        assert_eq!(output, "안녕하세요!");
        assert_eq!(offsets, vec![0, 0, 0, 0, 0, 0]);
    }
    #[test]
    fn test_replace_non_printable_no_range_changes() {
        let input = b"Hello,\x00World!";
        let (output, offsets) = replace_non_printable(
            input,
            ReplaceNonPrintableConfig::default().tab_width(2),
        );
        assert_eq!(output, "Hello,␀World!");
        assert_eq!(offsets, vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

        let input = b"Hello,\x7FWorld!";
        let (output, offsets) = replace_non_printable(
            input,
            ReplaceNonPrintableConfig::default().tab_width(2),
        );
        assert_eq!(output, "Hello,␀World!");
        assert_eq!(offsets, vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    }

    fn test_proportion_of_printable_ascii_characters(
        input: &str,
        expected: f32,
    ) {
        let actual =
            proportion_of_printable_ascii_characters(input.as_bytes());
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_proportion_of_printable_ascii_characters_ascii() {
        test_proportion_of_printable_ascii_characters("Hello, World!", 1.0);
        test_proportion_of_printable_ascii_characters(
            "Hello, World!\x00",
            0.928_571_4,
        );
        test_proportion_of_printable_ascii_characters(
            "\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0A\x0B\x0C\x0D\x0E\x0F",
            0.0,
        );
    }

    fn test_preprocess_line(input: &str, expected: &str) {
        let (actual, _offset) = preprocess_line(input);
        assert_eq!(actual, expected, "input: {:?}", input);
    }

    #[test]
    fn test_preprocess_line_cases() {
        test_preprocess_line("Hello, World!", "Hello, World!");
        test_preprocess_line("Hello, World!\n", "Hello, World!");
        test_preprocess_line("Hello, World!\x00", "Hello, World!␀");
        test_preprocess_line("Hello, World!\x7F", "Hello, World!␀");
        test_preprocess_line("Hello, World!\u{FEFF}", "Hello, World!␀");
        test_preprocess_line(&"a".repeat(400), &"a".repeat(300));
    }

    #[test]
    fn test_make_match_string_printable() {
        let entry = Entry::new("Hello, World!".to_string())
            .with_match_indices(&[0, 7]);
        let (printable, match_indices) = make_result_item_printable(&entry);
        assert_eq!(printable, "Hello, World!");
        assert_eq!(match_indices, vec![(0, 1), (7, 8)]);

        let entry = Entry::new("Hello,\tWorld!".to_string())
            .with_match_indices(&[0, 10]);
        let (printable, match_indices) = make_result_item_printable(&entry);
        assert_eq!(printable, "Hello,    World!");
        assert_eq!(match_indices, vec![(0, 1), (13, 14)]);

        let entry = Entry::new("Hello,\nWorld!".to_string())
            .with_match_indices(&[0, 6]);
        let (printable, match_indices) = make_result_item_printable(&entry);
        assert_eq!(printable, "Hello,World!");
        assert_eq!(match_indices, vec![(0, 1), (6, 6)]);

        let entry = Entry::new("Hello, World!".to_string());
        let (printable, match_indices) = make_result_item_printable(&entry);
        assert_eq!(printable, "Hello, World!");
        assert_eq!(match_indices, vec![]);

        let entry =
            Entry::new("build.rs".to_string()).with_match_indices(&[0, 7]);
        let (printable, match_indices) = make_result_item_printable(&entry);
        assert_eq!(printable, "build.rs");
        assert_eq!(match_indices, vec![(0, 1), (7, 8)]);

        let entry =
            Entry::new("a\tbcd".repeat(65)).with_match_indices(&[0, 330]);
        let (printable, match_indices) = make_result_item_printable(&entry);
        assert_eq!(printable.len(), 480);
        assert_eq!(match_indices, vec![(0, 1)]);

        let entry =
            Entry::new("ジェ abc".to_string()).with_match_indices(&[0, 2]);
        let (printable, match_indices) = make_result_item_printable(&entry);
        assert_eq!(printable, "ジェ abc");
        assert_eq!(match_indices, vec![(0, 1), (2, 3)]);
    }
}
