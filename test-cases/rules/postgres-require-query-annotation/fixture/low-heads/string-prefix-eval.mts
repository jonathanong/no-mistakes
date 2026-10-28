import { read } from "@bench/db";
function dynamic() {
  let sql = "/* dynamic */ SELECT 1";
  sql += " LIMIT 1";
  eval("sql = 'SELECT 2'");
  read(sql); // low: false
}
