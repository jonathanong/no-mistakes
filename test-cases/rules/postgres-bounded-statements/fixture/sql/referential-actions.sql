-- Only directly targeted rows are judged; cascades can affect many related rows.
DELETE FROM accounts WHERE id = $1;
UPDATE accounts SET id = $2 WHERE id = $1;
DELETE FROM orders WHERE id = $1;
DELETE FROM accounts;
