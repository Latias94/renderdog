pub const WORKSPACE_RENDERDOC_REPLAY_VERSION: Option<&str> =
    option_env!("RENDERDOG_SYS_WORKSPACE_REPLAY_VERSION");

pub fn workspace_renderdoc_replay_version() -> Option<&'static str> {
    WORKSPACE_RENDERDOC_REPLAY_VERSION
}

pub fn normalize_renderdoc_version(value: &str) -> Option<String> {
    let mut parts: Vec<String> = Vec::new();
    let mut current = String::new();

    for ch in value.chars() {
        if ch.is_ascii_digit() {
            current.push(ch);
            continue;
        }

        if !current.is_empty() {
            parts.push(std::mem::take(&mut current));
            if parts.len() == 2 {
                break;
            }
        }
    }

    if !current.is_empty() && parts.len() < 2 {
        parts.push(current);
    }

    if parts.len() >= 2 {
        Some(format!("{}.{}", parts[0], parts[1]))
    } else {
        None
    }
}

pub fn renderdoc_versions_match(lhs: &str, rhs: &str) -> bool {
    match (
        normalize_renderdoc_version(lhs),
        normalize_renderdoc_version(rhs),
    ) {
        (Some(lhs), Some(rhs)) => lhs == rhs,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::{normalize_renderdoc_version, renderdoc_versions_match};

    #[test]
    fn normalize_renderdoc_version_extracts_major_minor() {
        assert_eq!(
            normalize_renderdoc_version("RenderDoc v1.43 loaded"),
            Some("1.43".to_string())
        );
        assert_eq!(
            normalize_renderdoc_version("1.43"),
            Some("1.43".to_string())
        );
        assert_eq!(normalize_renderdoc_version("unknown"), None);
    }

    #[test]
    fn renderdoc_versions_match_uses_normalized_major_minor() {
        assert!(renderdoc_versions_match("v1.43", "1.43"));
        assert!(!renderdoc_versions_match("v1.42", "1.43"));
        assert!(!renderdoc_versions_match("custom-build", "1.43"));
    }
}
