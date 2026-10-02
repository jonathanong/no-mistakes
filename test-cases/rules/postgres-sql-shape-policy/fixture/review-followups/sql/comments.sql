SELECT id FROM accounts
WHERE id NOT /* audited */ IN (SELECT account_id FROM bans);
