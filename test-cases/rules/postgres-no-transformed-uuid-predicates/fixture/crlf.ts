import { query } from "db";
// Keep physical CRLF so cooked template LF normalization remains tested.
query(`SELECT 1 FROM accounts
WHERE id::text = $1`);
const saved = `SELECT 1 FROM accounts
WHERE CAST(id AS text) = $1`;
query(saved);
query(`SELECT 1 FROM accounts
WHERE id = $1::uuid`);
