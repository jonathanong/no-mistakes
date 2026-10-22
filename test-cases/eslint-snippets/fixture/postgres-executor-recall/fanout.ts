import { query } from "@example/db";
import * as db from "@example/db";
const { query: destructured } = db;
const alias = query;
const forward = (sql) => query(sql);
function declared(sql) { return query(sql); }
Promise.all(items.map(() => destructured("SELECT 1")));
Promise.all(items.map(() => alias?.("SELECT 1")));
Promise.all(items.map(() => query.call(null, "SELECT 1")));
Promise.all(items.map(() => query[`apply`](null, ["SELECT 1"])));
Promise.all(items.map(() => forward("SELECT 1")));
Promise.all(items.map(() => declared("SELECT 1")));
Promise.all(items.map(() => db["query"]("SELECT 1")));
Promise.all(items.map(() => (0, alias)("SELECT 1")));
Promise.all(items.map(alias));
Promise.all(items.map(destructured));
// Passing a static namespace member is the same executor as its const alias.
Promise.all(items.map(db.read));
Promise.all(items.map(db[`query`]));
