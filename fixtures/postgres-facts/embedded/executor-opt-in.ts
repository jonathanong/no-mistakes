import { query, read } from '@example/db';
// Only the unscoped explicit query selector admits these generic member calls.
query('SELECT 1');
read('SELECT 2');
client.query('SELECT 3');
client['query']('SELECT 4');
(client.query as (sql: string) => unknown)('SELECT 5');
client.query!('SELECT 6');
