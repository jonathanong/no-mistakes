SELECT app.count(active) > 0 FROM orders;
SELECT COUNT(*) OVER () > 0 FROM orders;
SELECT id FROM accounts WHERE NOT (id NOT IN (SELECT account_id FROM bans));
SELECT id FROM accounts WHERE NOT NOT (id IN (SELECT account_id FROM bans));
