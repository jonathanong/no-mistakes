declare const flag: string;
declare const other: string;

const s = require(flag);

async function conditional() {
  return flag ? s.sql`SELECT 1` : other;
}

import sqlNs = require("@example/sql");
sqlNs = flag;

async function written() {
  return sqlNs.sql`SELECT 1`;
}
