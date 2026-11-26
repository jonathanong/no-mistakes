SELECT
  o.id,
  (SELECT a1.display_name FROM accounts a1 WHERE a1.id = o.account_id) AS account_name,
  -- no-mistakes-disable-next-line postgres-no-duplicate-keyed-lookups
  (SELECT a2.tier FROM accounts a2 WHERE a2.id = o.account_id) AS account_tier,
  (SELECT a3.region FROM accounts a3 WHERE a3.id = o.account_id) AS account_region -- no-mistakes-disable-line postgres-no-duplicate-keyed-lookups
FROM orders o;

SELECT
  o.id,
  p1.display_name,
  p2.avatar_url
FROM orders o
LEFT JOIN profiles p1 ON p1.id = o.owner_id
LEFT JOIN profiles p2 ON p2.id = o.owner_id;
