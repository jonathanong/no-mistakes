const pg = require("pg");
const { ["Pool"]: Pool } = pg;
const pool = new Pool();
pool.on("error", console.error);

function localRequire(require) {
  const { Pool } = require("pg");
  const Alias = require("pg").Pool;
  new Alias();
  return new Pool();
}

function localPool(Pool) {
  return new Pool();
}

function localNamespace(pg) {
  const Alias = pg.Pool;
  new Alias();
  return new pg.Pool();
}

const MemberPool = pg.Pool;
const memberPool = new MemberPool();
memberPool.on("error", console.error);

const { connection: { Pool: UnknownPool } } = require("pg");
new UnknownPool();

module.exports = { pool, memberPool, localRequire, localPool, localNamespace };
