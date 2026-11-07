const { query } = require("@example/db");
const SCRIPT_ROWS = [{ key: "user", table: "users" }];
function scriptRow(key) {
  const selected = SCRIPT_ROWS.find((entry) => entry.key === key);
  if (!selected) throw new Error("Unknown relation");
  return selected;
}
async function run() {
  const selected = scriptRow("user");
  await query(`SELECT * FROM ${selected.table}`);
}
