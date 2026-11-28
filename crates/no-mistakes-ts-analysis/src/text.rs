pub use no_mistakes_ts_facts::syntax_helpers::byte_offset_to_line;

pub const SKIP_DIRS: &[&str] = &[
    "node_modules",
    "dist",
    ".git",
    ".next",
    "coverage",
    "fixtures",
    "target",
    "build",
];

pub fn is_skipped_dir(name: &str) -> bool {
    SKIP_DIRS.contains(&name)
}

pub fn line_number(source: &str, start: u32) -> usize {
    byte_offset_to_line(source, start as usize) as usize
}
