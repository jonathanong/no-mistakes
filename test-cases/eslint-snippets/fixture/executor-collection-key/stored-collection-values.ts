import { query, type QueryExecutor } from "@example/db";

declare function mutate(value: unknown): void;

const holder = new Map<string, unknown>();
const cache = new Map<QueryExecutor, unknown>();
holder.set("cache", cache);
mutate(holder);
cache.get(query);

const callHolder = new Map<string, unknown>();
const callCache = new Map<QueryExecutor, unknown>();
Map.prototype.set.call(callHolder, "cache", callCache);
callCache.get(query);

const applyHolder = new Map<string, unknown>();
const applyCache = new Map<QueryExecutor, unknown>();
Map.prototype.set.apply(applyHolder, ["cache", applyCache]);
applyCache.get(query);

const dynamicApplyHolder = new Map<string, unknown>();
const dynamicApplyCache = new Map<QueryExecutor, unknown>();
const applyArgs: [string, unknown] = ["cache", dynamicApplyCache];
Map.prototype.set.apply(dynamicApplyHolder, applyArgs);
dynamicApplyCache.get(query);

const setHolder = new Set<unknown>();
const setCache = new Map<QueryExecutor, unknown>();
setHolder.add(setCache);
setCache.get(query);

const weakHolder = new WeakMap<object, unknown>();
const weakCache = new Map<QueryExecutor, unknown>();
weakHolder.set({}, weakCache);
weakCache.get(query);

const weakSetHolder = new WeakSet<object>();
const weakSetCache = new Map<QueryExecutor, unknown>();
weakSetHolder.add(weakSetCache);
weakSetCache.get(query);

// Executor keys stay in a key slot; stored executor values remain outside this rule.
const keyHolder = new Map<QueryExecutor, string>();
keyHolder.set(query, "stored value");

const keyOwner = new Map<unknown, unknown>();
const keyCache = new Map<QueryExecutor, unknown>();
keyOwner.set(keyCache, "stored as a key");
mutate(keyOwner);
keyCache.get(query);

const callKeyOwner = new Map<unknown, unknown>();
const callKeyCache = new Map<QueryExecutor, unknown>();
Map.prototype.set.call(callKeyOwner, callKeyCache, "stored as a key");
mutate(callKeyOwner);
callKeyCache.get(query);

const applyKeyOwner = new Map<unknown, unknown>();
const applyKeyCache = new Map<QueryExecutor, unknown>();
Map.prototype.set.apply(applyKeyOwner, [applyKeyCache, "stored as a key"]);
mutate(applyKeyOwner);
applyKeyCache.get(query);
