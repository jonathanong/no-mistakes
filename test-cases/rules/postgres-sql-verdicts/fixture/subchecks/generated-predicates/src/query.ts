import { query } from "@example/db";
// Every call certifies each enabled sibling independently.
query("SELECT id FROM items WHERE created_at > now()");
query("SELECT a.id FROM items a JOIN items b ON a.created_at = b.created_at");
query("SELECT id FROM items ORDER BY created_at");
query("SELECT id FROM items WHERE id = $1");
query("SELECT id FROM missing WHERE created_at > now()");
