import { query } from "@example/db";
const result = await query(`SELECT schema FROM tenants`);
const [first] = result.rows;
first.schema = "public";
first.schema = { toString() { return process.env.SCHEMA; } };
query(`SELECT id FROM ${first.schema}.users`);
// A fresh object is not a finite scalar: its coercion can read the environment.
