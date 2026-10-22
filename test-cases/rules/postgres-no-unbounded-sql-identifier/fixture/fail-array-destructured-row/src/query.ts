import { query } from "@example/db";
const result = await query(`SELECT id FROM tenants`);
const [first] = result.rows;
const { id } = first;
query(`SELECT id FROM ${id}.users`);
