use std::path::PathBuf;

pub fn resolve_paths(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    paths.into_iter().map(|path| resolve_path(path)).collect()
}

pub fn resolve_path(path: PathBuf) -> PathBuf {
    path.canonicalize().unwrap()
}

pub fn slice_at_char_boundary(s: &str, byte_index: usize) -> &str {
    let mut last_valid_index = 0;

    for (i, _) in s.char_indices() {
        if i > byte_index {
            break;
        }
        last_valid_index = i;
    }

    &s[..last_valid_index]
}
