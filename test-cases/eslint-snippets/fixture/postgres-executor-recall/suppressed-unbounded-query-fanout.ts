import { query } from "@example/db";
const alias = query;
const TABLES = { orders: "orders" }; const kind = input;
// no-mistakes-disable-next-line postgres-no-unbounded-query-fanout
Promise.all(items.map(() => alias("SELECT 1")));
Promise.all(items.map(() => alias("SELECT 1"))); // no-mistakes-disable-line postgres-no-unbounded-query-fanout
