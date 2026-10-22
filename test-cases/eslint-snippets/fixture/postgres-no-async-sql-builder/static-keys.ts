import * as db from "@example/db";

// `other` and `...rest` are not the `sql` binding. The property filter must skip them.
const { other, sql, ...rest } = db;
const { sql: renamed } = db;
const { ["sql"]: literal } = db;
const { [`sql`]: template } = db;

async function fromMember() {
  return db["sql"]`SELECT 1`;
}
async function fromTemplate() {
  return db[`sql`]`SELECT 1`;
}
async function fromDestructure() {
  return sql`SELECT 1`;
}
async function fromRename() {
  return renamed`SELECT 1`;
}
async function fromLiteralKey() {
  return literal`SELECT 1`;
}
async function fromTemplateKey() {
  return template`SELECT 1`;
}

read(fromMember());
read(fromTemplate());
read(fromDestructure());
read(fromRename());
read(fromLiteralKey());
read(fromTemplateKey());
