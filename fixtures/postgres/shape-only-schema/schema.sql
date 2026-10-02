CREATE TABLE accounts (
  id int
);

SELECT id
FROM accounts
WHERE id NOT IN (SELECT id FROM bans);
