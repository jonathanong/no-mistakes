import { query } from "db";
// Aggregate values and parameter-side calls leave the UUID lookup operand bare.
query("SELECT email FROM accounts GROUP BY email HAVING COUNT(DISTINCT id) > 1");
query("SELECT email FROM accounts GROUP BY email HAVING lower(MAX(id)::text) = $1");
query("SELECT 1 FROM accounts WHERE id >= uuid_from_timestamp($1)");
query("SELECT 1 FROM accounts WHERE uuid_from_timestamp($1) <= id");
query("SELECT email FROM accounts GROUP BY email HAVING COUNT(DISTINCT id::text) > 1");
query("SELECT email FROM accounts GROUP BY email HAVING COUNT(fn(id)) > 1");
// FILTER predicates and same-name user scalar functions must still report.
query("SELECT email FROM accounts GROUP BY email HAVING COUNT(id) FILTER (WHERE id::text = $1) > 1");
query("SELECT email FROM accounts GROUP BY email HAVING app.min(id) = $1");
query("SELECT 1 FROM accounts WHERE fn(id) >= $1");
query("SELECT 1 FROM accounts WHERE id::text = $1");
query("SELECT 1 FROM accounts WHERE id >= fn(token)");
query("SELECT 1 FROM accounts WHERE fn(token) <= id");
