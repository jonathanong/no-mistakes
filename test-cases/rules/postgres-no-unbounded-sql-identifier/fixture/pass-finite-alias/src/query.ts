import { query } from "@example/db";
function schemaOf(value) {
  switch (value) { case "app": return "app"; default: throw new Error("unsupported"); }
}
const a = schemaOf(process.env.SCHEMA);
const b = a;
query(`SELECT id FROM ${b}.users`);
