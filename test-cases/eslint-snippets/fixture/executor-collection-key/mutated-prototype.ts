import { query, type QueryExecutor } from "@example/db";

const map = new Map<QueryExecutor, unknown>();
Map.prototype["get"] = function () {
  return undefined;
};
map.get(query);
