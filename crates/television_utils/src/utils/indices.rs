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
