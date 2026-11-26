const s = require("@example/sql");
const { sql: q } = require("@example/sql");
import t = require("@example/sql");

async function fromRequire() {
  return s.sql`SELECT 1`;
}

async function fromDestructure() {
  return q`SELECT 1`;
}

async function fromEquals() {
  return t.sql`SELECT 1`;
}
