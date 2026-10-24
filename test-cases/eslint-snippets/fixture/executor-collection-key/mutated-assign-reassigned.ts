import { query, type QueryExecutor } from "@example/db";

let assign = Object.assign;
assign = (target) => target;
const map = new Map<QueryExecutor, unknown>();
assign(map, { get: () => undefined });
map.get(query);
