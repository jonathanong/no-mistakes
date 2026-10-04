import { query } from "@example/db";
query(`SELECT id FROM orders WHERE created_at > $1`);
