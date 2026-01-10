pub(crate) fn default_host() -> String {
    "localhost".to_string()
}

pub(crate) fn default_frames() -> u32 {
    1
}

pub(crate) fn default_timeout_s() -> u32 {
    60
}

pub(crate) fn default_true() -> bool {
    true
}

pub(crate) fn default_max_results() -> Option<u32> {
    Some(200)
}
