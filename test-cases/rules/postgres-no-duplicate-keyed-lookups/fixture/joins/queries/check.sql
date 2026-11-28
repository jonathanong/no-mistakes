SELECT
  o.id,
  a1.display_name,
  a2.tier
FROM orders o
LEFT JOIN accounts a1 ON a1.id = o.account_id
LEFT JOIN accounts a2 ON o.account_id = a2.id;

SELECT
  o.id,
  p.display_name,
  (SELECT p2.avatar_url FROM profiles p2 WHERE p2.id = o.owner_id) AS owner_avatar
FROM orders o
LEFT JOIN profiles p ON p.id = o.owner_id;

SELECT
  o.id,
  a1.display_name,
  a2.tier
FROM orders o
LEFT JOIN accounts a1 ON a1.id = pg_catalog.md5(CAST(o.raw_account_id IS NULL AS text))::uuid
LEFT JOIN accounts a2 ON a2.id = CAST(pg_catalog.md5(CAST(o.raw_account_id IS NULL AS text)) AS uuid);
