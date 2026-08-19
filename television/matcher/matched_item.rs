/// A matched item.
///
/// This contains the matched item, the dimension against which it was matched,
/// represented as a string, and the indices of the matched characters.
///
/// The indices are character positions within the matched string.
#[derive(Debug, Clone)]
pub struct MatchedItem<I>
where
    I: Sync + Send + Clone + 'static,
{
    /// The matched item.
    pub inner: I,
    /// The dimension against which the item was matched (as a string).
    pub matched_string: String,
    /// The UTF-32 char indices of the matched characters.
    pub match_indices: Vec<u32>,
}

impl<I> MatchedItem<I>
where
    I: Sync + Send + Clone + 'static,
{
    /// Create a new `MatchedItem` from the given frizbee `Match` and the
    /// dimension against which it was matched.
    pub fn new(
        inner: I,
        matched_string: String,
        match_indices: Vec<u32>,
    ) -> Self {
        Self {
            inner,
            matched_string,
            match_indices,
        }
    }
}
/// Convert sorted UTF-8 byte offsets to UTF-32 char indices
pub fn byte_indices_to_char_indices(
    haystack: &str,
    byte_indices: Vec<u32>,
) -> Vec<u32> {
    if haystack.is_ascii() {
        return byte_indices;
    }

    let mut byte_indices = byte_indices.into_iter().peekable();
    let mut char_indices = Vec::with_capacity(byte_indices.size_hint().0);
    let mut last_char_index = None;

    for (char_index, (char_start, ch)) in haystack.char_indices().enumerate() {
        let char_end = char_start + ch.len_utf8();

        while byte_indices
            .peek()
            .is_some_and(|byte_index| (*byte_index as usize) < char_end)
        {
            let byte_index = byte_indices.next().unwrap() as usize;
            if byte_index < char_start {
                continue;
            }

            let char_index = u32::try_from(char_index)
                .expect("matched character index does not fit in u32");
            if last_char_index != Some(char_index) {
                char_indices.push(char_index);
                last_char_index = Some(char_index);
            }
        }
    }

    char_indices
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::matcher::{Matcher, SortStrategy};

    #[test]
    fn byte_indices_to_char_indices_collapses_multibyte_scalars() {
        assert_eq!(
            byte_indices_to_char_indices("xxإنyy", vec![2, 3, 4, 5]),
            vec![2, 3]
        );
        assert_eq!(
            byte_indices_to_char_indices("é😀x", vec![0, 1, 6]),
            vec![0, 2]
        );
        assert_eq!(
            byte_indices_to_char_indices("_______😀x", vec![7, 8, 9, 10, 11]),
            vec![7, 8]
        );
    }

    #[test]
    fn matcher_results_expose_character_indices_for_unicode_matches() {
        let mut matcher = Matcher::new(SortStrategy::Index, 1);
        let injector = matcher.injector();
        injector.push((), "xxإنyy".to_string());

        matcher.find("إن");
        matcher.wait_for_idle();

        let results = matcher.results(1, 0);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].match_indices, vec![2, 3]);
    }
}
