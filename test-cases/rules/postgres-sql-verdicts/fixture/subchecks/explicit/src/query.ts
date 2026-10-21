import { query } from "@example/db";
// Every call certifies each enabled sibling independently.
query("SELECT * FROM accounts");
query("UPDATE accounts SET name = 'x' RETURNING *");
query("SELECT id FROM accounts");
query("SELECT * FROM missing");
