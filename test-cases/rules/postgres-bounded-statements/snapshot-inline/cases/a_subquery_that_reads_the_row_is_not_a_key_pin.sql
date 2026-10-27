DELETE FROM accounts a WHERE a.id IN (SELECT a.id);
