SELECT id FROM accounts a WHERE NOT EXISTS (SELECT 1 FROM bans b WHERE b.account_id = a.id);
