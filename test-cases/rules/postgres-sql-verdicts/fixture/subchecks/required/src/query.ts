import { query } from "@example/db";
// Every call certifies each enabled sibling independently.
query("SELECT id FROM events");
query("SELECT id FROM events WHERE kind = 1 AND owner_id = $1 AND account_id = $2");
query("SELECT id FROM events WHERE owner_id = $1 AND account_id = $2");
query("SELECT id FROM events WHERE kind = 1 AND account_id = $1");
query("SELECT id FROM events WHERE kind = 1 AND owner_id = $1");
