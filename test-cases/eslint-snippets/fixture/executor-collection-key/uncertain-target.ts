import { query, type QueryExecutor } from "@example/db";

declare const unknownTarget: any;
declare const source: object;
Object.assign(unknownTarget, source);
Object.defineProperty(unknownTarget, "get", { value: undefined });
const map = new Map<QueryExecutor, unknown>();
map.get(query);
