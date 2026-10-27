DELETE FROM accounts WHERE id IN (SELECT account_id FROM external_accounts);
