import { query } from "db";
// Boolean/CASE conditions do not become transformed UUID values through the surrounding wrapper.
query("SELECT 1 FROM accounts WHERE CAST(id = $1 AS boolean)");
query("SELECT 1 FROM accounts WHERE COALESCE(id = $1, false)");
query("SELECT 1 FROM accounts WHERE CAST(CASE WHEN id = $1 THEN email ELSE email END AS text) = $2");
query("SELECT 1 FROM accounts WHERE CAST(CASE id WHEN $1 THEN email ELSE email END AS text) = $2");
// Actual transformed UUID values must still report under those same wrappers.
query("SELECT 1 FROM accounts WHERE CAST(id::text = $1 AS boolean)");
query("SELECT 1 FROM accounts WHERE COALESCE(id::text = $1, false)");
query("SELECT 1 FROM accounts WHERE CAST(CASE WHEN id = $1 THEN id ELSE $1::uuid END AS text) = $2");
query("SELECT 1 FROM accounts WHERE COALESCE((uuid_extract_timestamp(id) + INTERVAL '1 day') > $1, false)");
