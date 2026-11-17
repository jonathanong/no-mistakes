//! Cli postgres integration tests, linked in one harness.

include!("suite_cli_postgres_part1.rs");

include!("suite_cli_postgres_part2.rs");

#[path = "cli_postgres_snapshot_identifier_issue_3501.rs"]
mod cli_postgres_snapshot_identifier_issue_3501;

#[path = "cli_postgres_snapshot_issue_3505.rs"]
mod cli_postgres_snapshot_issue_3505;

#[path = "cli_postgres_snapshot_grouping_unanalyzable.rs"]
mod cli_postgres_snapshot_grouping_unanalyzable;

#[path = "cli_postgres_snapshot_lock_written_rows_catalog.rs"]
mod cli_postgres_snapshot_lock_written_rows_catalog;

#[path = "cli_postgres_snapshot_frozen_followups.rs"]
mod cli_postgres_snapshot_frozen_followups;
