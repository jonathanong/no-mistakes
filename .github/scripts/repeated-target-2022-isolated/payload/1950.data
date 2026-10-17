INSERT INTO accounts VALUES (1, 'ON CONFLICT; VALUES');
INSERT INTO public."Accounts" AS a ("ID", value) VALUES (2, 'select'), (3, 'do update') ON CONFLICT DO NOTHING;
INSERT INTO accounts DEFAULT VALUES;
INSERT INTO accounts (id) SELECT id FROM (SELECT id FROM source_accounts) AS nested;
INSERT INTO accounts (id) VALUES (1) ON CONFLICT (id) DO NOTHING;
INSERT INTO accounts VALUES (1) ON CONFLICT ON CONSTRAINT "Accounts_pkey" DO NOTHING;
-- The comment and quoted keyword must not change statement boundaries.
INSERT INTO accounts AS a (id, value) VALUES (1, 'WHERE DO') ON /* predicate */ CONFLICT (id) WHERE id > 0 DO UPDATE SET value = EXCLUDED.value, id = a.id WHERE a.id > 0;
INSERT INTO accounts VALUES (1) ON CONFLICT (id) DO UPDATE SET a = accounts.a, b = 42, c = $1, d = unknown.d, e = clock_timestamp(), f = (EXCLUDED.f)::text;
INSERT INTO accounts VALUES (1) ON CONFLICT (id) WHERE id IS NOT NULL DO NOTHING;
INSERT INTO accounts VALUES (1) ON CONFLICT (id) DO UPDATE SET (a, b) = (1, 2);
insert into accounts values (1) on conflict (id) where id > 0 do nothing;
INSERT INTO accounts AS a VALUES (1) ON CONFLICT (id) DO UPDATE SET id = accounts.id RETURNING id;
INSERT INTO accounts AS excluded VALUES (1) ON CONFLICT (id) DO UPDATE SET id = excluded.id;
