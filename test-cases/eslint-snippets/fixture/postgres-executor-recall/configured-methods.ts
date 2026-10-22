import * as db from "@example/db";
// These are configured executor names, not Function.prototype helpers.
db.call("BEGIN");
db["apply"]("COMMIT");
const { call: execute } = db;
execute("ROLLBACK");
Promise.all(items.map(db.call));
Promise.all(items.map(db[`apply`]));
