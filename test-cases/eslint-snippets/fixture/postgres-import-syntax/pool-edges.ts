declare const flag: string;

const dynamic = require(flag);
new dynamic.Pool();

const { Pool } = require(flag);
new Pool();

import pg = require("pg");
pg = dynamic;
new pg.Pool();

const missing = require();
new missing.Pool();

const extra = require("pg", flag);
new extra.Pool();

const {
  ns: { Pool: Nested },
} = { other: 1, ns: require("pg") };
new Nested();

const {
  gone: { Pool: Absent },
} = { other: 1 };
new Absent();
