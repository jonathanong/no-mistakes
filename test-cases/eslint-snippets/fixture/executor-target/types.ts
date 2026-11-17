export type QueryExecutor = (statement: string) => unknown;
export declare const read: QueryExecutor;
export declare const write: QueryExecutor;
