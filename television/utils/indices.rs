use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

const ELLIPSIS: &str = "…";
const ELLIPSIS_CHAR_WIDTH_U16: u16 = 1;
const ELLIPSIS_CHAR_WIDTH_U32: u32 = 1;
const ELLIPSIS_CHAR_WIDTH_USIZE: usize = 1;

/// Truncate a string to fit within a certain width, while keeping track of the
/// indices of the highlighted characters.
///
/// This will either truncate from the start or the end of the string, depending
/// on where the highlighted characters are.
///
/// This will take care of non-unit width characters such as emojis, or certain
/// CJK characters that are wider than a single character.
///
/// # Note
/// This function assumes that the highlighted ranges are sorted and non-overlapping.
///
/// # Examples
/// ```
/// use television::utils::indices::truncate_highlighted_string;
///
/// let s = "hello world";
/// let highlighted_ranges = vec![(0, 2), (4, 8), (10, 11)];
/// let max_width = 6;
/// let (truncated, ranges) = truncate_highlighted_string(
///     s,
///     &highlighted_ranges,
///     max_width,
/// );
///
/// assert_eq!(truncated, "…world");
/// assert_eq!(ranges, vec![(1, 3), (5, 6)]);
///
/// let s = "下地.mp3";
/// let highlighted_ranges = vec![(3, 5)];
/// let max_width = 5;
/// let (truncated, ranges) = truncate_highlighted_string(
///     s,
///     &highlighted_ranges,
///     max_width,
/// );
/// assert_eq!(truncated, "….mp3");
/// assert_eq!(ranges, vec![(2, 4)]);
/// ```
///
/// See unit tests for more examples.
pub fn truncate_highlighted_string<'a>(
    s: &'a str,
    highlighted_ranges: &'a [(u32, u32)],
    max_width: u16,
) -> (String, Vec<(u32, u32)>) {
    let str_width = s.width();

    if str_width <= max_width as usize {
        return (s.to_string(), highlighted_ranges.to_vec());
    }

    let last_highlighted_char_index =
        (highlighted_ranges.last().unwrap_or(&(0, 0)).1 as usize)
            // ranges are exclusive on the right
            .saturating_sub(1);
    let width_to_last_highlighted_char = s
        .chars()
        .take(last_highlighted_char_index + 1)
        .fold(0, |acc, c| acc + c.width().unwrap_or(0));

    // if the string isn't highlighted, or all highlighted characters are within the max index,
    // simply truncate it from the right and add an ellipsis
    if highlighted_ranges.is_empty()
        // is the last highlighted char index within the first "`max_width` of" characters?
        || width_to_last_highlighted_char < max_width as usize
    {
        let mut cumulative_width = 0;
        return (
            s.chars()
                .take_while(|c| {
                    cumulative_width += c.width().unwrap_or(0);
                    cumulative_width
                        <= max_width.saturating_sub(ELLIPSIS_CHAR_WIDTH_U16)
                            as usize
                })
                .collect::<String>()
                + ELLIPSIS,
            highlighted_ranges.to_vec(),
        );
    }

    // otherwise, if the last highlighted char index is within the last "max width" chars of the
    // string, truncate it from the left and add an ellipsis at the beginning
    // |<------- str_width ------->|
    //           |<-- max_width -->|
    // |--------> start_width_offset - 1 (for the ellipsis)
    let start_width_offset = str_width.saturating_sub(max_width as usize)
        + ELLIPSIS_CHAR_WIDTH_USIZE;
    if width_to_last_highlighted_char > start_width_offset {
        let mut truncated_width = str_width;
        let chars_to_skip = s
            .chars()
            .take_while(|c| {
                if truncated_width >= max_width as usize {
                    truncated_width -= c.width().unwrap_or(0);
                    true
                } else {
                    false
                }
            })
            .count();
        let truncated_string =
            s.chars().skip(chars_to_skip).collect::<String>();
        return (
            ELLIPSIS.to_string() + &truncated_string,
            highlighted_ranges
                .iter()
                .map(|(start, end)| {
                    (
                        start.saturating_sub(
                            u32::try_from(chars_to_skip).unwrap(),
                        ) + ELLIPSIS_CHAR_WIDTH_U32,
                        end.saturating_sub(
                            u32::try_from(chars_to_skip).unwrap(),
                        ) + ELLIPSIS_CHAR_WIDTH_U32,
                    )
                })
                .filter(|(start, end)| start != end)
                .collect(),
        );
    }

    // otherwise, try to put the last highlighted character towards the end of the truncated string and
    // truncate from both sides to fit the max width
    let start_width_offset =
        // 0123456789012
        //    ^^  ^     highlights
        // a long string
        // -------x     width to last highlighted char: 7
        //              max width = 4
        //      … s…    truncated string
        //      <--> 4
        width_to_last_highlighted_char.saturating_sub(max_width.saturating_sub(2*ELLIPSIS_CHAR_WIDTH_U16) as usize);

    let mut cumulated_width = 0;
    let chars_to_skip = s
        .chars()
        .take_while(|c| {
            if cumulated_width < start_width_offset {
                cumulated_width += c.width().unwrap_or(0);
                true
            } else {
                false
            }
        })
        .count();

    (
        ELLIPSIS.to_string()
            + &s.chars()
                .skip(chars_to_skip)
                .take(max_width.saturating_sub(2 * ELLIPSIS_CHAR_WIDTH_U16)
                    as usize)
                .collect::<String>()
            + ELLIPSIS,
        highlighted_ranges
            .iter()
            .map(|(start, end)| {
                (
                    start
                        .saturating_sub(u32::try_from(chars_to_skip).unwrap())
                        + ELLIPSIS_CHAR_WIDTH_U32,
                    end.saturating_sub(u32::try_from(chars_to_skip).unwrap())
                        + ELLIPSIS_CHAR_WIDTH_U32,
                )
            })
            .filter(|(start, end)| start != end)
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    #[test]
    /// string:         themes/solarized-light.toml
    /// highlights:                            ----
    /// max width:      ---------------------------
    /// result:         themes/solarized-light.toml
    /// expected:                              ----
    fn test_truncate_hightlighted_string_no_op() {
        let s = "themes/solarized-light.toml";
        let highlighted_ranges = vec![(23, 27)];
        let max_width = 27;
        let (truncated, ranges) = super::truncate_highlighted_string(
            s,
            &highlighted_ranges,
            max_width,
        );
        assert_eq!(truncated, s);
        assert_eq!(ranges, highlighted_ranges);
    }

    #[test]
    /// string:     hello world
    /// highlights:
    /// max width:  -----
    /// result:     hell…
    fn test_truncate_hightlighted_string_no_highlight() {
        let s = "hello world";
        let highlighted_ranges = vec![];
        let max_width = 5;
        let (truncated, ranges) = super::truncate_highlighted_string(
            s,
            &highlighted_ranges,
            max_width,
        );
        assert_eq!(truncated, "hell…");
        assert_eq!(ranges, highlighted_ranges);
    }

    #[test]
    /// string:     hello world
    /// highlights: -----
    /// max width:  ----------
    /// result:     hello wor…
    fn test_truncate_hightlighted_string_highlights_fit_left() {
        let s = "hello world";
        let highlighted_ranges = vec![(0, 5)];
        let max_width = 10;
        let (truncated, ranges) = super::truncate_highlighted_string(
            s,
            &highlighted_ranges,
            max_width,
        );
        assert_eq!(truncated, "hello wor…");
        assert_eq!(ranges, highlighted_ranges);
    }

    #[test]
    /// string:     hello world
    /// highlights: --  ----  -
    ///             he  o wo  d
    /// max width:  ------
    ///                  ------
    /// result:          …world
    fn test_truncate_highlighted_string_highlights_right() {
        let s = "hello world";
        let highlighted_ranges = vec![(0, 2), (4, 8), (10, 11)];
        let max_width = 6;
        let (truncated, ranges) = super::truncate_highlighted_string(
            s,
            &highlighted_ranges,
            max_width,
        );

        assert_eq!(truncated, "…world");
        assert_eq!(ranges, vec![(1, 3), (5, 6)]);
    }

    #[test]
    /// string:     下地.mp3
    /// highlights:      ---
    /// max width:     -----
    /// result:        ….mp3
    fn test_truncate_hightlighted_string_highlights_right_wide_chars() {
        let s = "下地.mp3";
        let highlighted_ranges = vec![(3, 5)];
        let max_width = 5;
        let (truncated, ranges) = super::truncate_highlighted_string(
            s,
            &highlighted_ranges,
            max_width,
        );
        assert_eq!(truncated, "….mp3");
        assert_eq!(ranges, vec![(2, 4)]);
    }

    #[test]
    /// string:         themes/solarized-light.toml
    /// highlights:                            ----
    /// max width:       --------------------------
    /// result:          …emes/solarized-light.toml
    /// expected:                              ----
    fn test_truncate_highlighted_string_truncate_left() {
        let s = "themes/solarized-light.toml";
        let highlighted_ranges = vec![(23, 27)];
        let max_width = 26;
        let (truncated, ranges) = super::truncate_highlighted_string(
            s,
            &highlighted_ranges,
            max_width,
        );

        assert_eq!(truncated, "…emes/solarized-light.toml");
        assert_eq!(ranges, vec![(22, 26)]);
    }

    #[test]
    fn ellipsis_len() {
        assert_eq!(
            super::ELLIPSIS.chars().count(),
            super::ELLIPSIS_CHAR_WIDTH_U16 as usize
        );
    }

    #[test]
    /// string:         themes/solarized-light.toml
    /// highlights:                ---
    /// max width:              ------- 7
    /// result:                 …lariz…
    /// expected:                  ---
    fn test_truncate_highlighted_string_truncate_middle() {
        let s = "themes/solarized-light.toml";
        let highlighted_ranges = vec![(11, 14)];
        let max_width = 7;
        let (truncated, ranges) = super::truncate_highlighted_string(
            s,
            &highlighted_ranges,
            max_width,
        );

        assert_eq!(truncated, "…lariz…");
        assert_eq!(ranges, vec![(3, 6)]);
    }

    #[test]
    // 0123456789012
    //    ^^  ^     highlights
    // a long string
    // ---------x   last highlighted char index: 9
    //              max width = 4
    //      … s…    truncated string
    //      <--> 4
    fn test_truncate_highlighted_string_truncate_both_ends() {
        let s = "a long string";
        let highlighted_ranges = vec![(3, 5), (7, 8)];
        let max_width = 4;
        let (truncated, ranges) = super::truncate_highlighted_string(
            s,
            &highlighted_ranges,
            max_width,
        );

        assert_eq!(truncated, "… s…");
        assert_eq!(ranges, vec![(2, 3)]);
    }

    #[test]
    /// string:     下地下地abc下地下地
    /// highlights:         ---
    /// max width:         -----
    /// result:            …abc…
    fn test_truncate_hightlighted_string_truncate_both_ends_wide_chars() {
        let s = "下地下地abc下地下地";
        let highlighted_ranges = vec![(4, 7)];
        let max_width = 5;
        let (truncated, ranges) = super::truncate_highlighted_string(
            s,
            &highlighted_ranges,
            max_width,
        );
        assert_eq!(truncated, "…abc…");
        assert_eq!(ranges, vec![(1, 4)]);
    }
}
