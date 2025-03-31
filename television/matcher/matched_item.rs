/// A matched item.
///
/// This contains the matched item, the dimension against which it was matched,
/// represented as a string, and the indices of the matched characters.
///
/// The indices are pairs of `(start, end)` where `start` is the index of the
/// first character in the match, and `end` is the index of the character after
/// the last character in the match.
#[derive(Debug, Clone)]
pub struct MatchedItem<I>
where
    I: Sync + Send + Clone + 'static,
{
    /// The matched item.
    pub inner: I,
    /// The dimension against which the item was matched (as a string).
    pub matched_string: String,
    /// The indices of the matched characters.
    pub match_indices: Vec<u32>,
}
