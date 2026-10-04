import { query } from '@example/db';
const shared = "SELECT id FROM orders\nWHERE id NOT IN (SELECT id FROM bans)";
query(shared);
const below =
  `SELECT id FROM orders
WHERE (SELECT COUNT(*) FROM events) > 0`;
query(below);
query("SELECT id FROM orders\nWHERE (SELECT COUNT(*) FROM events) = 0");
query(`SELECT id FROM orders \
WHERE id NOT IN (SELECT id FROM bans)`);
// no-mistakes-disable-next-line postgres-sql-shape-policy
query("SELECT id FROM orders\nWHERE id NOT IN (SELECT id FROM bans)");
