import { query } from "@example/db";
query(`SELECT id FROM users WHERE name = ${"app_" + process.env.SCHEMA}`);
