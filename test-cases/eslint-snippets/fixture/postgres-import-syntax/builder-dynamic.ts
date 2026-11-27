declare const flag: string;
const s = require(flag);

async function load() {
  return s.sql`SELECT 1`;
}
