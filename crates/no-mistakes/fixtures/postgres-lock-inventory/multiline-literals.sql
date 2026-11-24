SELECT id FROM jobs WHERE tag = 'first
second' FOR UPDATE;
SELECT pg_advisory_xact_lock(hashtext($tag$first
second$tag$));
