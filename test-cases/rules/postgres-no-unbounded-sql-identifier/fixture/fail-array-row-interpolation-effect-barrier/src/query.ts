import { query } from "@example/db";
const result = await query(`SELECT schema FROM tenants`);
const [first] = result.rows;
declare function mutate(row: unknown): string;
first.schema = "public";
query(`SELECT ${mutate(first)} FROM ${first.schema}.users`);
// Another interpolation may mutate the row before the identifier is read.
