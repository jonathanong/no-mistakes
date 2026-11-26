declare const flag: string;

import db = require("net");
db = flag;
db.read("BEGIN");

import known = require("@example/db");
known = flag;
known.read("SELECT 1");

declare const read: (sql: string) => Promise<unknown>;
read("BEGIN");
