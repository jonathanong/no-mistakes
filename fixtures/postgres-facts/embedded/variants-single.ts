import { query, sql } from "@example/db";
query("SELECT id FROM users LIMIT 1");
const q = sql`SELECT id FROM users WHERE id = ${id}`;
query(q);
let mutable = "SELECT id FROM users";
query(mutable); // Legacy nonbranching mutable call retains its opaque policy.
query(`SELECT id
FROM users
LIMIT 1`);
