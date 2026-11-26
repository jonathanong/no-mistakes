SELECT
  o.id,
  buyer.display_name AS buyer_name,
  seller.display_name AS seller_name
FROM orders o
LEFT JOIN accounts buyer ON buyer.id = o.buyer_id
LEFT JOIN accounts seller ON seller.id = o.seller_id;

SELECT
  o.id,
  a1.display_name,
  a2.tier
FROM orders o
LEFT JOIN accounts a1 ON a1.id = o.raw_id::uuid
LEFT JOIN accounts a2 ON a2.id = o.raw_id::text;

SELECT
  o.id,
  billing.postal_code AS billing_zip,
  shipping.postal_code AS shipping_zip
FROM orders o
LEFT JOIN addresses billing ON billing.account_id = o.account_id AND billing.kind = 'billing'
LEFT JOIN addresses shipping ON shipping.account_id = o.account_id AND shipping.kind = 'shipping';

SELECT
  o.id,
  a.display_name,
  (SELECT COUNT(*) FROM accounts a2 WHERE a2.id = o.account_id) AS match_count
FROM orders o
LEFT JOIN accounts a ON a.id = o.account_id;

SELECT
  o.id,
  a.display_name
FROM orders o
LEFT JOIN accounts a ON a.id = o.account_id
WHERE EXISTS (SELECT 1 FROM accounts a2 WHERE a2.id = o.account_id);

SELECT
  o.id,
  CASE
    WHEN o.use_short_name
      THEN (SELECT a1.short_name FROM accounts a1 WHERE a1.id = o.account_id)
    ELSE (SELECT a2.display_name FROM accounts a2 WHERE a2.id = o.account_id)
  END AS label
FROM orders o;

SELECT
  o.id,
  a1.display_name,
  a2.tier
FROM orders o
LEFT JOIN accounts a1 ON a1.id = gen_random_uuid()
LEFT JOIN accounts a2 ON a2.id = gen_random_uuid();

SELECT
  o.id,
  a1.display_name,
  a2.tier
FROM orders o
LEFT JOIN accounts a1 ON a1.id = coalesce(o.override_id, uuidv4())
LEFT JOIN accounts a2 ON a2.id = coalesce(o.override_id, uuidv4());

SELECT
  o.id,
  a1.display_name,
  a2.tier
FROM orders o
LEFT JOIN accounts a1 ON a1.id = coalesce(o.override_id, uuidv7())
LEFT JOIN accounts a2 ON a2.id = coalesce(o.override_id, uuidv7());

SELECT
  o.id,
  (SELECT a1.display_name FROM accounts a1 WHERE a1.id = app.next_lookup_id(o.id)) AS first_name,
  (SELECT a2.tier FROM accounts a2 WHERE a2.id = app.next_lookup_id(o.id)) AS second_tier
FROM orders o;

SELECT
  o.id,
  a1.display_name,
  a2.tier
FROM orders o
LEFT JOIN accounts a1 ON a1.id = o.raw_id::"Key"
LEFT JOIN accounts a2 ON a2.id = o.raw_id::"key";

SELECT
  o.id,
  (SELECT a1.display_name FROM accounts a1 WHERE a1.id = o.account_id LIMIT 0) AS none_name,
  (SELECT a2.tier FROM accounts a2 WHERE a2.id = o.account_id LIMIT 1) AS one_tier
FROM orders o;

SELECT
  o.id,
  a.display_name,
  b.tier
FROM orders o
LEFT JOIN accounts a(id, other) ON a.id = o.account_id
LEFT JOIN accounts b(other, id) ON b.id = o.account_id;

SELECT
  o.id,
  a.display_name,
  (SELECT json_object_agg_strict(a2.code, a2.tier) FROM accounts a2 WHERE a2.id = o.account_id) AS strict_map,
  (SELECT jsonb_object_agg_unique(a3.code, a3.tier) FROM accounts a3 WHERE a3.id = o.account_id) AS unique_map
FROM orders o
LEFT JOIN accounts a ON a.id = o.account_id;

UPDATE orders o
SET
  account_id = o.next_account_id,
  cached_name = (SELECT a1.display_name FROM accounts a1 WHERE a1.id = o.account_id)
RETURNING (SELECT a2.tier FROM accounts a2 WHERE a2.id = o.account_id);
