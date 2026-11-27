#[derive(Debug, Default, Clone, Copy)]
pub struct CheckFactStats {
    pub files_discovered: usize,
    pub files_parsed: usize,
    pub parse_errors: usize,
}

pub fn unproven_member_query_warning(count: u32) -> Option<String> {
    (count > 0).then(|| {
        format!(
            "warning: {count} member .query calls with SQL-looking text weren't proven to be executors; see docs/migrations/explicit-postgres-executors.md"
        )
    })
}
