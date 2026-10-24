import { query, type QueryExecutor } from "@example/db";

Reflect.defineProperty(Map.prototype, "get", { value: (_key: QueryExecutor) => undefined });
const map = new Map<QueryExecutor, unknown>();
map.get(query);

Reflect.setPrototypeOf(Set.prototype, {});
const set = new Set<QueryExecutor>();
set.has(query);

const define = Reflect.defineProperty;
define(Map.prototype, "has", { value: (_key: QueryExecutor) => false });
const aliasedMap = new Map<QueryExecutor, unknown>();
aliasedMap.has(query);

const setPrototype = Reflect.setPrototypeOf;
setPrototype(Set.prototype, {});
const aliasedSet = new Set<QueryExecutor>();
aliasedSet.has(query);

const deleteProperty = Reflect.deleteProperty;
deleteProperty(WeakMap.prototype, "get");
const aliasedWeakMap = new WeakMap<QueryExecutor, unknown>();
aliasedWeakMap.get(query);

const dynamicName: string = "has";
Reflect.defineProperty(Set.prototype, dynamicName, { value: (_key: QueryExecutor) => false });
set.has(query);
