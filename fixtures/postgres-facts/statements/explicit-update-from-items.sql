SELECT 1 FROM accounts AS a WHERE a.id = $1 FOR UPDATE OF a;
SELECT 1 FROM accounts AS a, account_events AS e
WHERE a.id = $1 FOR UPDATE OF a;
