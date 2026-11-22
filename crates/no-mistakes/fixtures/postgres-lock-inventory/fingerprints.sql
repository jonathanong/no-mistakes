SELECT id FROM "Jobs" WHERE label = 'a b' FOR UPDATE;

select id from "Jobs" where label='a b'
  FOR /* trailing lock */ UPDATE;
SELECT id FROM "Jobs" WHERE label = 'ab' FOR UPDATE;
SELECT id FROM "jobs" WHERE label = 'a b' FOR UPDATE;
SELECT id FROM "Jobs" WHERE label = 'a b' FOR NO KEY UPDATE;
SELECT pg_try_advisory_lock(4), pg_try_advisory_lock(4);
SELECT pg_try_advisory_lock(4), pg_try_advisory_lock(4);
