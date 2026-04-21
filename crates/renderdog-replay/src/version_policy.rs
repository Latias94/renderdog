#[cfg(any(feature = "cxx-replay", test))]
pub(crate) const WORKSPACE_RENDERDOC_REPLAY_VERSION: Option<&str> =
    option_env!("RENDERDOG_REPLAY_WORKSPACE_VERSION");

#[cfg(any(feature = "cxx-replay", test))]
pub(crate) fn workspace_renderdoc_replay_version() -> Option<&'static str> {
    WORKSPACE_RENDERDOC_REPLAY_VERSION
}

#[cfg(any(feature = "cxx-replay", test))]
pub(crate) fn normalize_renderdoc_version(value: &str) -> Option<String> {
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

#[cfg(any(feature = "cxx-replay", test))]
pub(crate) fn renderdoc_versions_match(lhs: &str, rhs: &str) -> bool {
    match (
        normalize_renderdoc_version(lhs),
        normalize_renderdoc_version(rhs),
    ) {
        (Some(lhs), Some(rhs)) => lhs == rhs,
        _ => false,
    }
}
