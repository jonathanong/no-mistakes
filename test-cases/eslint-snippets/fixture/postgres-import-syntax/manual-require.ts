const db = require("@example/db");
const { read: r } = require("@example/db");
import e = require("@example/db");

db.read("BEGIN");
r("BEGIN");
e.read("BEGIN");
