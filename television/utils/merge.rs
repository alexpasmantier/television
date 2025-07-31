/// Merge strategies for `Option`
pub mod option {
    /// Overwrite `left` with `right` only if `right` is `Some`.
    pub fn overwrite_if_some<T>(left: &mut Option<T>, right: Option<T>) {
        if let Some(new) = right {
            *left = Some(new);
        }
    }

    /// If both `left` and `right` are `Some`, recursively merge the two.
    /// Otherwise, fall back to `overwrite_some`.
    pub fn recurse<T: merge::Merge>(left: &mut Option<T>, right: Option<T>) {
        if let Some(new) = right {
            if let Some(original) = left {
                original.merge(new);
            } else {
                overwrite_if_some(left, Some(new));
            }
        }
    }
}
