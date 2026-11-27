function require(name: string) {
  return { Pool: class Pool {} };
}

const { Pool } = require("pg");
new Pool();

import net = require("net");
new net.Pool();
