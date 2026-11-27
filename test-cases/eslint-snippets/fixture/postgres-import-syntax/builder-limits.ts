declare const flag: string;

import sqlNs = require("net");
sqlNs = flag;

async function written() {
  return sqlNs.sql`SELECT 1`;
}
