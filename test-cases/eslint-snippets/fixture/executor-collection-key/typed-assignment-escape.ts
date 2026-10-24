import { query, type QueryExecutor } from "@example/db";

declare function mutate(value: unknown): void;

const cache = new Map<QueryExecutor, unknown>();
let leaked: Map<QueryExecutor, unknown>;
leaked = cache;
mutate(leaked);
cache.get(query);

const fieldCache = new Map<QueryExecutor, unknown>();
const holder: { cache?: Map<QueryExecutor, unknown> } = {};
holder.cache = fieldCache;
mutate(holder);
fieldCache.get(query);

const andCache = new Map<QueryExecutor, unknown>();
let andLeak: any;
andLeak &&= andCache;
andCache.get(query);

const orCache = new Map<QueryExecutor, unknown>();
let orLeak: any;
orLeak ||= orCache;
orCache.get(query);

const nullishCache = new Map<QueryExecutor, unknown>();
let nullishLeak: any;
nullishLeak ??= nullishCache;
nullishCache.get(query);
