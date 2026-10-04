import { query } from "@example/db";

// Shape lines must be rebased to this declaration.
const statement = `SELECT id FROM accounts
WHERE id NOT IN (SELECT account_id FROM bans)
AND (SELECT COUNT(*) FROM orders) = 0`;
query(statement);
