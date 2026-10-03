import psql from '@example/db';
await psql.query('SELECT id FROM users WHERE id = $1');
