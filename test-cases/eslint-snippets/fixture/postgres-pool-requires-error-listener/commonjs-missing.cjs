const { Pool } = require("pg");
new Pool();

const { Pool: RenamedPool } = require("pg");
new RenamedPool();

const pg = require("pg");
new pg.Pool();

const { connection: { Pool: NestedPool } } = { connection: pg };
new NestedPool();

new (require("@example/pg-compatible").Pool)();
