import { query } from '@example/db';
// Conflict-only preparation must not project unrelated lock/column/star sites.
query('SELECT id, count(*) FROM accounts WHERE id > 0 ORDER BY id FOR UPDATE');
query('INSERT INTO accounts (id) SELECT id FROM accounts ORDER BY id ON CONFLICT (id) DO NOTHING');
