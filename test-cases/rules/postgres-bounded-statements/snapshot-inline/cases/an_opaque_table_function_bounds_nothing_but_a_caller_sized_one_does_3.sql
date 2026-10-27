DELETE FROM accounts WHERE id = ANY(ARRAY(SELECT id FROM accounts));
