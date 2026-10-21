import { query } from "@example/db";
// Each call keeps sibling obligation outcomes independent.
query("SELECT id FROM accounts");
query("UPDATE accounts SET name = 'x'");
query("DELETE FROM accounts");
query("SELECT id FROM accounts WHERE id = $1");
query("UPDATE accounts SET name = 'x' WHERE id = $1");
query("DELETE FROM accounts WHERE id = $1");
query("SELECT id FROM missing");
