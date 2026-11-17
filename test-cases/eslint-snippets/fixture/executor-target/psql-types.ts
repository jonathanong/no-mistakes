export type QueryExecutor = (statement: string) => unknown;
export type TransactionQuery = QueryExecutor;
export type PoolClient = QueryExecutor;
export type QueryOptions = { query: QueryExecutor };
export declare const readPool: QueryExecutor;
export declare const writePool: QueryExecutor;
export declare function beginTransaction(options: { client: QueryExecutor }): unknown;
export declare function beginBoundedTransaction(options: { client: QueryExecutor }): unknown;
