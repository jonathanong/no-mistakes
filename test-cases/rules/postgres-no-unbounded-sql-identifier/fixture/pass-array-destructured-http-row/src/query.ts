import { query } from "@example/db";
const result = await fetch("https://example.test");
const [first] = result.rows;
const { schema } = first;
query(`SELECT id FROM ${schema}.users`);
