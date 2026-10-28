import { query } from '@example/db';

// PostgreSQL accepts one ANALYZE statement with several relation targets.
query('ANALYZE households, household_members, individuals, users');
query('ANALYZE stories, rss_feed_items, rss_feed_item_sources, rss_feeds, urls');
// A trailing comma still leaves the SQL unanalyzable.
query('ANALYZE households,');
query('DROP TABLE users');
