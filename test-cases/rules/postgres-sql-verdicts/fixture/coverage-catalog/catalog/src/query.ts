import { query } from "@example/db";

query("SELECT id FROM events");
query("SELECT id FROM events WHERE account_id = $1 AND owner_id = $2");
// The unknown join peer may own an otherwise missing partition-key column.
query("SELECT e.id FROM events e JOIN mystery m ON m.id = e.id");
query("SELECT id FROM ordinary");
query("SELECT id FROM missing_table");
query("SELECT id FROM unkeyed");
query("SELECT id FROM expression_events");
