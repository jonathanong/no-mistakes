import { query } from "@db";

query("SELECT id FROM orders WHERE account_id = $1");

query("SELECT id FROM orders");

query("UPDATE orders SET status = 'x'");

query("DELETE FROM orders");
