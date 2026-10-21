const sql = require("sql-template-strings");
const database = require("@bench/db");

let statement = sql`SELECT id FROM orders`;
const retained = sql`SELECT id FROM orders`;
function callback() {
  return retained;
}
const callback0 = () => 0;
const callback1 = () => 1;
const callback2 = () => 2;
const callback3 = () => 3;
const scalar0 = 0;
const scalar1 = 1;
const scalar2 = 2;
const scalar3 = 3;
const scalar4 = 4;
const scalar5 = 5;
const scalar6 = 6;
const scalar7 = 7;
const scalar8 = 8;
const scalar9 = 9;
const scalar10 = 10;
const scalar11 = 11;
const scalar12 = 12;
const scalar13 = 13;
const scalar14 = 14;
const scalar15 = 15;
const scalar16 = 16;
const scalar17 = 17;
const scalar18 = 18;
const scalar19 = 19;

// One arm mutates the shared SQL builder; scalar/function bindings stay unchanged.
const selected = flag
  ? statement.append(" WHERE enabled = true")
  : statement.append(" WHERE enabled = false");
database.query(retained);
database.query(callback());
