mod composer;
mod gemfile;
mod gomod;
mod pipfile;
mod toml_lock;

pub(crate) use composer::{parse as parse_composer, parse_checked as parse_composer_checked};
pub(crate) use gemfile::{parse as parse_gemfile, parse_checked as parse_gemfile_checked};
pub(crate) use gomod::{parse as parse_gomod, parse_checked as parse_gomod_checked};
pub(crate) use pipfile::{parse as parse_pipfile, parse_checked as parse_pipfile_checked};
pub(crate) use toml_lock::{parse as parse_toml, parse_checked as parse_toml_checked, TomlKind};

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod edge_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod source_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod tests;

fn source_fingerprint(pin: &str, selector: &str, value: Option<&str>) -> String {
    match value.filter(|value| !value.is_empty()) {
        Some(value) => format!("{pin}#{selector}={value}"),
        None => pin.to_string(),
    }
}

fn hash_fingerprint<'a>(hashes: impl Iterator<Item = &'a str>) -> String {
    let mut hashes: Vec<_> = hashes.filter(|hash| !hash.is_empty()).collect();
    hashes.sort_unstable();
    hashes.dedup();
    hashes.join(",")
}
