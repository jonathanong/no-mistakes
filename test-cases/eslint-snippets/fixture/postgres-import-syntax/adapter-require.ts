const db = require("@example/db");
const { read: r } = require("@example/db");
import e = require("@example/db");

function run(query: (sql: string) => Promise<unknown>) {
  return query;
}

run(db.read);
run(r);
run(e.read);
