SELECT a.id
FROM accounts a
JOIN authorizations z ON z.id = a.authorization_id AND z.status = $2
WHERE a.id = $1;
