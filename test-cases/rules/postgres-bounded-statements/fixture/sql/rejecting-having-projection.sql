-- HAVING removes the implicit group before SELECT-list or ORDER BY expansion.
DELETE FROM accounts WHERE id IN (SELECT unnest(get_all_ids()) HAVING false);
DELETE FROM accounts WHERE id IN (SELECT unnest(get_all_ids()) HAVING NULL);
DELETE FROM accounts WHERE id IN (SELECT unnest(get_all_ids()) HAVING true);
DELETE FROM accounts WHERE id IN (SELECT unnest(get_all_ids()) HAVING $1);
DELETE FROM accounts WHERE id IN (SELECT unnest(get_all_ids()) FROM orders HAVING false);
DELETE FROM accounts WHERE id IN (SELECT unnest(get_all_ids()) FROM orders WHERE orders.id = $1 HAVING false);
DELETE FROM accounts WHERE id IN (SELECT unnest(get_all_ids()) HAVING true LIMIT 1);
DELETE FROM accounts WHERE id IN (SELECT 1 HAVING false ORDER BY unnest(get_all_ids()));
DELETE FROM accounts WHERE id IN (SELECT 1 HAVING NULL ORDER BY unnest(get_all_ids()));
DELETE FROM accounts WHERE id IN (SELECT 1 HAVING true ORDER BY unnest(get_all_ids()));
DELETE FROM accounts WHERE id IN (SELECT unnest(get_all_ids()) HAVING app.truth());
DELETE FROM accounts WHERE id IN (SELECT unnest(get_all_ids()) FROM orders HAVING NULL);
DELETE FROM accounts WHERE id IN (SELECT unnest(get_all_ids()) HAVING true AND (NULL));
-- no-mistakes-disable-next-line postgres-bounded-statements
DELETE FROM accounts WHERE id IN (SELECT unnest(get_all_ids()) HAVING true);
DELETE FROM accounts WHERE id IN (SELECT unnest(get_all_ids()) HAVING false AND $1);
DELETE FROM accounts WHERE id IN (SELECT unnest(get_all_ids()) HAVING true AND $1);
