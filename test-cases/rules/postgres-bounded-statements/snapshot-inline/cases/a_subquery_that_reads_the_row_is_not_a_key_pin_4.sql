DELETE FROM accounts a WHERE a.id = (SELECT a.id);
