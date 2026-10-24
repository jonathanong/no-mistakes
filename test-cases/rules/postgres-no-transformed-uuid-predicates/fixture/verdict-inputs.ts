import { query } from "db";
query("SELECT 1 FROM accounts WHERE lower(email) = $1");
query("SELECT 1 FROM accounts a WHERE a.missing::text = $1");
query("SELECT 1 FROM accounts a JOIN orders o ON true WHERE id::text = $1");
query("SELECT 1 FROM accounts a WHERE a.id::text = $1 AND a.missing::text = $2");
query("SELECT 1 FROM accounts a WHERE a.missing::text = $1 AND a.id::text = $2");
