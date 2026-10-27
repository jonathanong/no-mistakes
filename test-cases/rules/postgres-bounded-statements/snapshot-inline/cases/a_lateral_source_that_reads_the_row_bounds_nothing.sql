SELECT * FROM accounts a JOIN LATERAL (SELECT a.id AS id) d ON a.id = d.id;
