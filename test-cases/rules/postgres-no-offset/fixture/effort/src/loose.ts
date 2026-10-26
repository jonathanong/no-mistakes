import { query } from '@example/db';
query('SELECT 1 OFFSET 1');
function local(query: (sql: string) => unknown) { query('SELECT 1 OFFSET 2'); }
globalQuery('SELECT 1 OFFSET 3');
other('SELECT 1 OFFSET 4');
