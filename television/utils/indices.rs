use super::strings::prev_char_boundary;

pub fn sep_name_and_value_indices(
    indices: &mut Vec<u32>,
    name_len: u32,
) -> (Vec<u32>, Vec<u32>, bool, bool) {
    let mut name_indices = Vec::new();
    let mut value_indices = Vec::new();
    let mut should_add_name_indices = false;
    let mut should_add_value_indices = false;

    for i in indices.drain(..) {
        if i < name_len {
            name_indices.push(i);
            should_add_name_indices = true;
        } else {
            value_indices.push(i - name_len);
            should_add_value_indices = true;
        }
    }

    name_indices.sort_unstable();
    name_indices.dedup();
    value_indices.sort_unstable();
    value_indices.dedup();

    (
        name_indices,
        value_indices,
        should_add_name_indices,
        should_add_value_indices,
    )
}

const ELLIPSIS: &str = "…";
const ELLIPSIS_CHAR_WIDTH_U16: u16 = 1;
const ELLIPSIS_BYTE_LEN_U32: u32 = 3;

/// Truncate a string to fit within a certain width, while keeping track of the
/// indices of the highlighted characters.
///
/// This will either truncate from the start or the end of the string, depending
/// on where the highlighted characters are.
///
/// NOTE: This function assumes that the highlighted ranges are sorted and non-overlapping.
pub fn truncate_highlighted_string<'a>(
    s: &'a str,
    highlighted_ranges: &'a [(u32, u32)],
    max_width: u16,
) -> (String, Vec<(u32, u32)>) {
    let (byte_positions, chars) =
        s.char_indices().unzip::<_, _, Vec<_>, Vec<_>>();

    if chars.len() <= max_width as usize {
        return (s.to_string(), highlighted_ranges.to_vec());
    }

    let max_byte_index = byte_positions[usize::from(max_width)];

    let last_highlighted_byte_index =
        highlighted_ranges.last().unwrap_or(&(0, 0)).1 as usize;

    // if the string isn't highlighted, or all highlighted characters are within the max byte index,
    // simply truncate it from the right and add an ellipsis
    if highlighted_ranges.is_empty()
        // is the last highlighted byte index within the max byte index?
        || last_highlighted_byte_index < max_byte_index - 1
    {
        return (
            s.chars()
                .take(
                    max_width.saturating_sub(ELLIPSIS_CHAR_WIDTH_U16) as usize
                )
                .collect::<String>()
                + ELLIPSIS,
            highlighted_ranges.to_vec(),
        );
    }

    // otherwise, if the last highlighted byte index is within the last "max width" bytes of the
    // string, truncate it from the left and add an ellipsis at the beginning
    let start_offset = (chars.len() + 1).saturating_sub(max_width as usize);
    let byte_offset = byte_positions[start_offset];
    if last_highlighted_byte_index > byte_offset {
        return (
            ELLIPSIS.to_string()
                + &s.chars().skip(start_offset).collect::<String>(),
            highlighted_ranges
                .iter()
                .map(|(start, end)| {
                    (
                        start.saturating_sub(
                            u32::try_from(byte_offset).unwrap(),
                        ) + ELLIPSIS_BYTE_LEN_U32,
                        end.saturating_sub(
                            u32::try_from(byte_offset).unwrap(),
                        ) + ELLIPSIS_BYTE_LEN_U32,
                    )
                })
                .filter(|(start, end)| start != end)
                .collect(),
        );
    }

    // otherwise, try to put the last highlighted character towards the end of the truncated string and
    // truncate from both sides to fit the max width
    let byte_offset =
        // note that we're using `max_width` here as a rough estimate to avoid more complex calculations
        // and then finding the closest character boundary
        prev_char_boundary(s, last_highlighted_byte_index.saturating_sub(max_width.saturating_sub(2) as usize));

    (
        ELLIPSIS.to_string()
            + &s[byte_offset..]
                .chars()
                .take(max_width.saturating_sub(2 * ELLIPSIS_CHAR_WIDTH_U16)
                    as usize)
                .collect::<String>()
            + ELLIPSIS,
        highlighted_ranges
            .iter()
            .map(|(start, end)| {
                (
                    start.saturating_sub(u32::try_from(byte_offset).unwrap())
                        + ELLIPSIS_BYTE_LEN_U32,
                    end.saturating_sub(u32::try_from(byte_offset).unwrap())
                        + ELLIPSIS_BYTE_LEN_U32,
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
        // the ellipsis is 3 bytes long
        assert_eq!(ranges, vec![(3, 5), (7, 8)]);
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
        assert_eq!(ranges, vec![(24, 28)]);
    }

    #[test]
    fn ellipsis_len() {
        assert_eq!(
            super::ELLIPSIS.chars().count(),
            super::ELLIPSIS_CHAR_WIDTH_U16 as usize
        );
        assert_eq!(
            super::ELLIPSIS.len(),
            super::ELLIPSIS_BYTE_LEN_U32 as usize
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
        assert_eq!(ranges, vec![(5, 8)]);
    }
}
