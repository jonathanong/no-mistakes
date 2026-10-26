import { query } from '@example/db';
// Keep physical CRLF: cooked template line endings are LF.
query(`SELECT 1
OFFSET 2`);
query(`SELECT 1
LIMIT 2`);
const saved = `SELECT 1
OFFSET 0`;
query(saved);
query(`SELECT 1
OFFSET \u0033`);
