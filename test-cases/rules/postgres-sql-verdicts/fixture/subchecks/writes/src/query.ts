import { query } from "@example/db";
// Every call certifies each enabled sibling independently.
query("UPDATE items SET created_at = now()");
query("UPDATE items SET updated_at = now()");
query("UPDATE items SET note = 'ok'");
query("UPDATE missing SET note = 'unknown'");
