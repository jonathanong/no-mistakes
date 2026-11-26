-- no-mistakes-disable-file postgres-no-duplicate-keyed-lookups
SELECT
  o.id,
  (SELECT a1.display_name FROM accounts a1 WHERE a1.id = o.account_id) AS account_name,
  (SELECT a2.tier FROM accounts a2 WHERE a2.id = o.account_id) AS account_tier
FROM orders o;
