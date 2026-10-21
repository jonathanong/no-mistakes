import { query } from "@example/db";
function schemaOf(key: string) {
  const tables: Record<string, string> = { app: "app" };
  return `${tables[key]}`; // An opaque computed interpolation is not a finite validator.
}
query(`SELECT id FROM ${schemaOf(process.env.SCHEMA)}.users`);
