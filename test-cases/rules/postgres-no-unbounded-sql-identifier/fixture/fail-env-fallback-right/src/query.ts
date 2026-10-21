import { query } from "@example/db";
declare const maybe: string | undefined;
const a = maybe ?? process.env.SCHEMA;
query(`SELECT id FROM ${a}.users`);
