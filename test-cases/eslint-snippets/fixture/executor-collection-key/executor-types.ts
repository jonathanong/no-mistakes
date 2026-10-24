export type QueryExecutor = (sql: string) => unknown;
export declare function query(sql: string): unknown;
export declare const collection: Map<QueryExecutor, unknown>;
