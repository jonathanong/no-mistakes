import { query, type QueryExecutor } from "@example/db";

export const exposed = new Map<QueryExecutor, unknown>();
exposed.get(query);

const named = new Map<QueryExecutor, unknown>();
export { named };
named.get(query);

const base = new Map<QueryExecutor, unknown>();
const exportedAlias = base;
export { exportedAlias };
base.get(query);

const defaultMap = new Map<QueryExecutor, unknown>();
export default defaultMap;
defaultMap.get(query);
