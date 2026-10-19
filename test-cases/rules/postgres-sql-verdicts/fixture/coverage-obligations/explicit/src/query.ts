import { query } from "@example/db";
// Whole-row functions are configured exemptions, including RETURNING.
query("SELECT row_to_json(a.*) FROM accounts a");
query("UPDATE accounts SET name = 'x' RETURNING row_to_json(accounts.*)");
query("SELECT * FROM accounts");
