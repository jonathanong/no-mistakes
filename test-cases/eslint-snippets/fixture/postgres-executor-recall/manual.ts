import { query } from "@example/db";
import * as db from "@example/db";
const { query: destructured } = db;
const alias = query;
const forward = (sql) => query(sql);
function declared(sql) { return query(sql); }
destructured("BEGIN");
alias?.("BEGIN");
query.call(null, "BEGIN");
query[`apply`](null, ["BEGIN"]);
forward("BEGIN");
declared("BEGIN");
db["query"]("BEGIN");
(0, alias)("BEGIN");
const { [`query`]: keyed } = db; keyed("BEGIN");
