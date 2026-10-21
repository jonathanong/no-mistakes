import { query } from "@example/db";
function schemaOf(value: string) {
  return value === "app" ? "app" : value === "archive" ? "archive" : "fallback";
}
query(`SELECT id FROM ${schemaOf(process.env.SCHEMA)}.users`);
