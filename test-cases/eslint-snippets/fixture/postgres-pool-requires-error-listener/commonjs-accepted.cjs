const pg = require("pg");
const { ["Pool"]: Pool } = pg;
const pool = new Pool();
pool.on("error", console.error);

function localRequire(require) {
  const { Pool } = require("pg");
  return new Pool();
}

function localPool(Pool) {
  return new Pool();
}

function localNamespace(pg) {
  return new pg.Pool();
}

const { connection: { Pool: UnknownPool } } = require("pg");
new UnknownPool();

module.exports = { pool, localRequire, localPool, localNamespace };
