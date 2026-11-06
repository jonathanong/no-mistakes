SELECT a.id
FROM accounts a
JOIN authorizations z ON z.status = a.status
WHERE a.id = $1;
