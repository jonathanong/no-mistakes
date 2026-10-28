-- PostgreSQL ANALYZE accepts several relations in one statement.
ANALYZE households, household_members, individuals, users;
ANALYZE stories,
    rss_feed_items, rss_feed_item_sources, rss_feeds, urls;
ANALYZE public.posts, "MixedCase"."Table";
ANALYZE DROP, public.posts;
