import { query } from "@example/db";
const response = await fetch("https://example.test");
const { rows } = response;
for (const row of rows) {
  query(`SELECT id FROM ${row.schema}.users`);
}
