import { query, type QueryExecutor } from "@example/db";

const map = new Map<QueryExecutor, unknown>();
const alias = map as Map<QueryExecutor, string>;
alias.get = (key) => "changed";
map.get(query);

const computed = new Map<QueryExecutor, unknown>();
computed["set"] = () => computed;
computed.set(query, "key");

const compound = new Map<QueryExecutor, unknown>();
compound.has ||= () => false;
compound.has(query);

const updated = new Map<QueryExecutor, unknown>();
updated.delete++;
updated.delete(query);

const destructured = new WeakMap<QueryExecutor, unknown>();
[destructured.get] = [() => undefined];
destructured.get(query);
