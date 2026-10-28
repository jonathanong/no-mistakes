import { query } from '@example/db';

// PostgreSQL accepts one ANALYZE statement with several relation targets.
query('ANALYZE households, household_members, individuals, users');
query('ANALYZE stories, rss_feed_items, rss_feed_item_sources, rss_feeds, urls');
query('ANALYZE VERBOSE households, household_members');
query('ANALYZE households(id), household_members(user_id), users');
query('ANALYZE VERBOSE public.households(id, "Name"), "Mixed"."Members"("userId")');
query('ANALYZE "VERBOSE"(id), public."VERBOSE"');
// A malformed target or unsupported option must retain an advisory or finding.
query('ANALYZE households,');
query('ANALYZE VERBOSE households(), users');
query('ANALYZE households(id), users DROP TABLE users');
query('ANALYZE (UNKNOWN_OPTION) households, users');
query('ANALYZE (VERBOSE) households, users');
// Valid maintenance must not hide a subsequent mutation in the same call.
query('ANALYZE VERBOSE households(id), household_members(user_id); DROP TABLE users');
query('DROP TABLE users');
