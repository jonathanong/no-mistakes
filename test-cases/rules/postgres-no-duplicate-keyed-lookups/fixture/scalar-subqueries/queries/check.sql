SELECT
  o.id,
  (SELECT display_name FROM accounts WHERE id = o.account_id) AS account_name,
  (SELECT tier FROM accounts WHERE id = o.account_id) AS account_tier,
  (SELECT p1.avatar_url FROM profiles p1 WHERE p1.id = o.owner_id) AS owner_avatar,
  (SELECT p2.handle FROM profiles p2 WHERE o.owner_id = p2.id) AS owner_handle
FROM orders o;
