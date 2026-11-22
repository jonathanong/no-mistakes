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
#[path = "cli_postgres_snapshot_lock_written_rows_followup.rs"]
mod cli_postgres_snapshot_lock_written_rows_followup;
#[path = "cli_postgres_snapshot_plain_helper_location.rs"]
mod cli_postgres_snapshot_plain_helper_location;

#[path = "cli_postgres_snapshot_indexed_identifier.rs"]
mod cli_postgres_snapshot_indexed_identifier;

#[path = "cli_postgres_snapshot_issue_3651.rs"]
mod cli_postgres_snapshot_issue_3651;

#[path = "cli_postgres_snapshot_require_query_annotation_destructured.rs"]
mod cli_postgres_snapshot_require_query_annotation_destructured;

#[path = "cli_postgres_snapshot_lock_written_rows_patterns.rs"]
mod cli_postgres_snapshot_lock_written_rows_patterns;

#[path = "cli_postgres_lock_inventory.rs"]
mod cli_postgres_lock_inventory;
#[path = "cli_postgres_lock_inventory_lifecycle.rs"]
mod cli_postgres_lock_inventory_lifecycle;
