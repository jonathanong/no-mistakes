#[cfg(target_os = "macos")]
fn accepts(value: &str) -> bool {
    value.is_empty()
}
