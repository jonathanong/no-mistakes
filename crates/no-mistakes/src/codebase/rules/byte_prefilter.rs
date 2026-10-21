//! Necessary-substring scans used before per-line regexes.
//!
//! A miss means the regex cannot match. A hit is not a finding.

use memchr::{memchr2, memmem};

pub(crate) fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
    memmem::Finder::new(needle).find(haystack).is_some()
}

pub(crate) fn is_ascii(haystack: &[u8]) -> bool {
    haystack.iter().all(|byte| byte.is_ascii())
}

pub(crate) fn contains_hex_run(haystack: &[u8], min_len: usize) -> bool {
    let mut run = 0usize;
    for &byte in haystack {
        if byte.is_ascii_hexdigit() {
            run += 1;
            if run >= min_len {
                return true;
            }
        } else {
            run = 0;
        }
    }
    false
}

pub(crate) fn contains_ascii_ignore_case(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() {
        return true;
    }
    let first_lo = needle[0].to_ascii_lowercase();
    let first_up = needle[0].to_ascii_uppercase();
    let mut offset = 0;
    while offset + needle.len() <= haystack.len() {
        let Some(relative) = memchr2(first_lo, first_up, &haystack[offset..]) else {
            return false;
        };
        let start = offset + relative;
        let Some(slice) = haystack.get(start..start + needle.len()) else {
            return false;
        };
        if slice.eq_ignore_ascii_case(needle) {
            return true;
        }
        offset = start + 1;
    }
    false
}

pub(crate) fn contains_any_ascii_ignore_case(haystack: &[u8], needles: &[Box<[u8]>]) -> bool {
    needles
        .iter()
        .any(|needle| contains_ascii_ignore_case(haystack, needle))
}

#[cfg(test)]
mod tests;
