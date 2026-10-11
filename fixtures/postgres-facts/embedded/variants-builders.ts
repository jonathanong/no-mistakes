import { query, sql } from "@example/db";
query(sql`SELECT id FROM users ${sql.join([])}`);
query(sql`SELECT id FROM users ${sql.join([sql`WHERE active`, sql` AND enabled`])}`);
query(sql`SELECT id FROM users ${sql("WHERE active")}`);
query(sql`SELECT id FROM users ${sql`WHERE active`.append(flag ? sql` AND enabled` : sql` AND verified`)}`);
query(sql`SELECT id FROM users ${sql.join([sql`WHERE active`, sql`enabled`], flag ? " AND " : " OR ")}`);
function where() { return sql`WHERE active`; }
query(sql`SELECT id FROM users ${where()}`);
query(sql`SELECT id FROM users ${sql["raw"]("WHERE active")}`);
query(sql`SELECT id FROM users ${sql["join"]([sql`WHERE active`, sql`AND enabled`], " ")}`);
query(sql`SELECT id FROM users ${sql`WHERE active`["append"](sql` AND enabled`)}`);
