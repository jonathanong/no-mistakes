import { query } from "@example/db";
const a = process.env.SCHEMA;
{
  const a = "app";
  const b = a;
  query(`SELECT id FROM ${b}.users`);
}
