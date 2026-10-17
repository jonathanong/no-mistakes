import { query, sql } from "@example/db";
query((flag ? null : "SELECT 1") ?? "SELECT 2");
query((flag ? sql`SELECT ${value}` : null) ?? sql`SELECT 3`);
query(sql`SELECT id FROM users ${(flag ? sql`WHERE active` : null) ?? sql`WHERE deleted`}`);
query(null); // A null executor argument is not a complete SQL statement.
query(flag ? null : "SELECT 4");
query((flag ? null : "SELECT 5") ?? unknownSql);
