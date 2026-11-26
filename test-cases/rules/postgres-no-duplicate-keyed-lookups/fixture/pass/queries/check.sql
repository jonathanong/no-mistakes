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
