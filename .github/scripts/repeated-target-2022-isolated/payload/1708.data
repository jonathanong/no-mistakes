import { query, sql } from "@example/db";
const statements = gate ? sql`
SET LOCAL statement_timeout = '1s';
SET SESSION lock_timeout TO '2s';
SET TIME ZONE 'UTC';
SELECT set_config('search_path', 'public', true), pg_sleep(0);
SELECT u.*, *, row_to_json(u.*),
       (SELECT x.* FROM inner_users x WHERE x.id = u.id LIMIT 1) AS nested
FROM users u JOIN accounts a ON u.account_id = a.id
WHERE NOT ((u.id > 0 AND a.id IN (1, 2)) OR u.id BETWEEN 3 AND 4)
ORDER BY u.id LIMIT 10 OFFSET 2;
SELECT id FROM users WHERE id NOT IN (SELECT id FROM accounts);
SELECT EXISTS(SELECT 1 FROM users UNION SELECT 1 FROM accounts);
SELECT (SELECT COUNT(*) FROM users) > 0;
SELECT u.name AS label FROM public.users u WHERE u.id <> 1 ORDER BY label, u.id + 1 LIMIT 1;
SELECT u.id FROM users u JOIN generate_series(1, 3) g ON u.id = g.id;
SELECT u.id FROM (users u JOIN accounts a ON u.id = a.user_id) WHERE u.id = 1;
SELECT id FROM users WHERE id = ${id} ORDER BY id FOR UPDATE;
UPDATE users u SET (active, name) = (true, 'new') FROM accounts a
WHERE (u.id = a.user_id AND NOT u.active = false)
RETURNING u.*, row_to_json(u.*);
DELETE FROM users AS u USING accounts AS a
WHERE u.id = a.user_id RETURNING *, row_to_json(u.*);
UPDATE users u SET active = true WHERE u.id = 1 RETURNING pg_catalog.row_to_json(record => u.*);
INSERT INTO users (id, active) VALUES (1, true)
ON CONFLICT (id) DO UPDATE SET active = false RETURNING *;
INSERT INTO public.users VALUES (2, true)
ON CONFLICT (id) DO NOTHING RETURNING users.*;
MERGE INTO users u USING accounts a ON u.id = a.id
WHEN MATCHED THEN UPDATE SET active = true
WHEN NOT MATCHED THEN INSERT (id, active) VALUES (a.id, true);
` : sql`
UPDATE alternative_users SET name = 'other' WHERE id = 1 RETURNING *;
SELECT id FROM alternative_users ORDER BY id LIMIT 1;
`;
query(sql`${statements}`);
