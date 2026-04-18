pub const AUTOPEER_BASE_PATH: &str = "/autopeer";

pub fn matches_autopeer_path(path: &str) -> bool {
    path == AUTOPEER_BASE_PATH
        || path
            .strip_prefix(AUTOPEER_BASE_PATH)
            .is_some_and(|suffix| suffix.is_empty() || suffix.starts_with('/'))
}

#[cfg(test)]
mod tests {
    use super::matches_autopeer_path;

    #[test]
    fn matches_base_autopeer_path() {
        assert!(matches_autopeer_path("/autopeer"));
    }

    #[test]
    fn matches_nested_autopeer_paths() {
        assert!(matches_autopeer_path("/autopeer/setup"));
        assert!(matches_autopeer_path("/autopeer/sessions/1"));
    }

    #[test]
    fn rejects_non_autopeer_paths() {
        assert!(!matches_autopeer_path("/"));
        assert!(!matches_autopeer_path("/protocols"));
        assert!(!matches_autopeer_path("/autopeering"));
    }
}
