DELETE FROM accounts a WHERE a.id = ANY (SELECT a.id);
