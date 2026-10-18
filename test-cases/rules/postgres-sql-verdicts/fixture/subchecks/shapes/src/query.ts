import { query } from "@example/db";
// Every call certifies each enabled sibling independently.
query("SELECT a.id FROM accounts a WHERE EXISTS (SELECT b.id FROM accounts b WHERE b.id = a.id UNION SELECT c.id FROM accounts c WHERE c.id = a.id)");
query("SELECT id FROM accounts WHERE id NOT IN (SELECT id FROM accounts)");
query("SELECT (SELECT count(*) FROM accounts) > 0");
query("SELECT id FROM accounts LIMIT 5");
query("SELECT id FROM accounts WHERE id > $1 ORDER BY id LIMIT $2");
query("SELECT pg_sleep(1)");
query("SELECT id FROM accounts WHERE id = $1");
