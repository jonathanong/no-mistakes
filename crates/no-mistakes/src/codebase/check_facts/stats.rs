#[derive(Debug, Default, Clone, Copy)]
pub struct CheckFactStats {
    pub files_discovered: usize,
    pub files_parsed: usize,
    pub parse_errors: usize,
}
