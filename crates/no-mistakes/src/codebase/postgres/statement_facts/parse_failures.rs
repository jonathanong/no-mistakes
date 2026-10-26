use super::{SqlFactSite, SqlStatementFileFacts};

impl SqlStatementFileFacts {
    /// Failure occurrences with a compatibility fallback for older/manual projections.
    pub fn parse_failure_sites(&self) -> impl Iterator<Item = (SqlFactSite, usize)> + '_ {
        self.parse_failures
            .iter()
            .enumerate()
            .filter(|_| self.parse_failed)
            .map(|(index, failure)| (SqlFactSite::ParseFailure(index), failure.line.max(1)))
            .chain(
                (self.parse_failed && self.parse_failures.is_empty())
                    .then_some((SqlFactSite::Origin, self.origin_line.max(1))),
            )
    }
}
