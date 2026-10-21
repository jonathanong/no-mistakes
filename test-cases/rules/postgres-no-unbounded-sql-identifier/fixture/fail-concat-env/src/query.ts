import { query } from "@example/db";
query(`SELECT id FROM ${"app_" + process.env.SCHEMA}.users`);
