import { query } from "@example/db";
import * as db from "@example/db";
const { query: destructured } = db;
const alias = query;
const forward = (sql) => query(sql);
function declared(sql) { return query(sql); }
const TABLES = { orders: "orders" };
const kind = input;
destructured(`SELECT ${TABLES[kind]}`);
alias?.(`SELECT ${TABLES[kind]}`);
query.call(null, `SELECT ${TABLES[kind]}`);
query[`apply`](null, [`SELECT ${TABLES[kind]}`]);
forward(`SELECT ${TABLES[kind]}`);
declared(`SELECT ${TABLES[kind]}`);
db["query"](`SELECT ${TABLES[kind]}`);
(0, alias)(`SELECT ${TABLES[kind]}`);
const { [`query`]: keyed } = db; keyed(`SELECT ${TABLES[kind]}`);
