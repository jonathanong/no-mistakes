//! Cli postgres integration tests, linked in one harness.

include!("suite_cli_postgres_part1.rs");

include!("suite_cli_postgres_part2.rs");

#[path = "cli_postgres_snapshot_identifier_issue_3501.rs"]
mod cli_postgres_snapshot_identifier_issue_3501;
