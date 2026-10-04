-- PostgreSQL rejects SRFs anywhere within CASE, so executable CASE expressions are scalar.
SELECT 1 FROM orders JOIN (SELECT CASE WHEN $1 THEN custom_scalar($2) ELSE $3 END AS id) d ON orders.id = d.id;
SELECT 1 FROM orders JOIN (SELECT CASE WHEN app.truth($1) THEN $2 ELSE $3 END AS id) d ON orders.id = d.id;
SELECT 1 FROM orders JOIN (SELECT CASE app.scalar($1) WHEN $2 THEN $3 ELSE $4 END AS id) d ON orders.id = d.id;
SELECT 1 FROM orders JOIN (SELECT CASE WHEN $1 THEN CASE WHEN $2 THEN app.scalar($3) ELSE $4 END ELSE $5 END AS id) d ON orders.id = d.id;
SELECT 1 FROM orders JOIN (SELECT CASE WHEN $1 THEN $2 ELSE app.scalar($3) END AS id) d ON orders.id = d.id;
SELECT 1 FROM orders JOIN (SELECT COALESCE(CASE WHEN $1 THEN app.scalar($2) ELSE $3 END, $4) AS id) d ON orders.id = d.id;
SELECT 1 FROM orders JOIN (SELECT custom_scalar($1) AS id) d ON orders.id = d.id;
SELECT 1 FROM orders JOIN (SELECT app.wrapper(CASE WHEN $1 THEN app.scalar($2) ELSE $3 END) AS id) d ON orders.id = d.id;
SELECT 1 FROM accounts JOIN (SELECT CASE WHEN $1 THEN app.scalar(o.id) ELSE o.id END AS id FROM orders o) d ON accounts.id = d.id;
SELECT 1 FROM accounts JOIN (SELECT CASE WHEN $1 THEN app.scalar(o.id) ELSE o.id END AS id FROM orders o WHERE o.id = $2) d ON accounts.id = d.id;
SELECT 1 FROM orders JOIN (SELECT custom_srf($1) AS id LIMIT 1) d ON orders.id = d.id;
SELECT 1 FROM orders JOIN (SELECT CASE WHEN $1 THEN $2 ELSE $3 END AS id, custom_srf($4)) d ON orders.id = d.id;
SELECT 1 FROM orders JOIN (SELECT CASE WHEN $1 THEN $2 ELSE $3 END AS id, unnest($4::uuid[])) d ON orders.id = d.id;
SELECT 1 FROM orders JOIN (SELECT CASE WHEN count(*) > 0 THEN app.scalar(count(*)) ELSE $1 END AS id FROM accounts) d ON orders.id = d.id;
