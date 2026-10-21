import { query } from "@example/db";
const e = process.env;
query(`SELECT id FROM ${e.SCHEMA}.users`);
