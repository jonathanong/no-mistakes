import { query } from "@example/db";

export function list() {
  const schema = process.env.SCHEMA as "app" | "public";
  return query(`SELECT id FROM ${schema}.users`);
}
