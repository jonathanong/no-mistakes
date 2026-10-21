import { query } from "@example/db";
// Every call certifies each enabled sibling independently.
query("SELECT id FROM accounts WHERE id IN ($1, $2) FOR UPDATE");
query("SELECT id FROM accounts WHERE id IN ($1, $2) ORDER BY name FOR UPDATE");
query("SELECT id FROM accounts WHERE id IN ($1, $2) ORDER BY id FOR UPDATE");
query("SELECT id FROM missing WHERE id IN ($1, $2) FOR UPDATE");
