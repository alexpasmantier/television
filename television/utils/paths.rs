use directories::UserDirs;
use std::path::{Path, PathBuf};

pub fn expand_tilde<P>(path: P) -> PathBuf
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    if path.starts_with("~") {
        let home = UserDirs::new()
            .map(|dirs| dirs.home_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("/"));
        home.join(path.strip_prefix("~").unwrap())
    } else {
        path.to_path_buf()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_tilde() {
        assert_eq!(
            expand_tilde("~/test").to_str().unwrap(),
            &format!("{}/test", UserDirs::new().unwrap().home_dir().display())
        );
        assert_eq!(expand_tilde("test").to_str().unwrap(), "test");
        assert_eq!(
            expand_tilde("/absolute/path").to_str().unwrap(),
            "/absolute/path"
        );
    }
}
