import { query } from "@example/db";
const result = await query(`SELECT schema FROM tenants`);
const [first] = result.rows;
first.schema = "public";
declare function mutate(row: unknown): void;
mutate(first);
query(`SELECT id FROM ${first.schema}.users`);
// An intervening call can replace the field, so adjacency is required for the finite proof.
