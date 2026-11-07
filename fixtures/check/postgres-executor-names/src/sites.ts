import { query, read as fetchRows, write, readPool } from '@example/db';
await query('SELECT 1');
await fetchRows('SELECT 2');
await write('SELECT 3');
await readPool('SELECT 4');
