#[cfg(any(feature = "cxx-replay", test))]
const VENDORED_WORKSPACE_RENDERDOC_REPLAY_VERSION: &str =
    include_str!("../vendor/renderdoc_replay_version.txt");

#[cfg(any(feature = "cxx-replay", test))]
pub(crate) fn workspace_renderdoc_replay_version() -> &'static str {
    option_env!("RENDERDOG_REPLAY_WORKSPACE_VERSION")
        .unwrap_or_else(|| VENDORED_WORKSPACE_RENDERDOC_REPLAY_VERSION.trim())
}

#[cfg(any(feature = "cxx-replay", test))]
pub(crate) fn normalize_renderdoc_version(value: &str) -> Option<String> {
    if let Some(version) = find_v_prefixed_version(value) {
        return Some(version);
    }

    first_two_numeric_components(value)
}

#[cfg(any(feature = "cxx-replay", test))]
fn find_v_prefixed_version(value: &str) -> Option<String> {
    for (idx, ch) in value.char_indices() {
        if ch != 'v' && ch != 'V' {
            continue;
        }

        if let Some(version) = parse_major_minor_prefix(&value[idx + ch.len_utf8()..]) {
            return Some(version);
        }
    }

    None
}

#[cfg(any(feature = "cxx-replay", test))]
fn parse_major_minor_prefix(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    let mut cursor = 0;

    while bytes
        .get(cursor)
        .is_some_and(std::primitive::u8::is_ascii_digit)
    {
        cursor += 1;
    }

    if cursor == 0 || bytes.get(cursor) != Some(&b'.') {
        return None;
    }

    let major = &value[..cursor];
    cursor += 1;
    let minor_start = cursor;

    while bytes
        .get(cursor)
        .is_some_and(std::primitive::u8::is_ascii_digit)
    {
        cursor += 1;
    }

    if cursor == minor_start {
        return None;
    }

    Some(format!("{major}.{}", &value[minor_start..cursor]))
}

#[cfg(any(feature = "cxx-replay", test))]
fn first_two_numeric_components(value: &str) -> Option<String> {
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
