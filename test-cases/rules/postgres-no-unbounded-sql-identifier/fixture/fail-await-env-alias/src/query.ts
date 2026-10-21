import { query } from "@example/db";
const a = await process.env.SCHEMA;
query(`SELECT id FROM ${a}.users`);
