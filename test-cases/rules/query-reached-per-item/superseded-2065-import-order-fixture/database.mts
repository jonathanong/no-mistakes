export type TransactionQuery = (sql: string) => Promise<unknown>;
export type QueryOptions = { query?: TransactionQuery };
export declare function beginTransaction(): Promise<TransactionQuery>;
export declare function write(sql: string): Promise<unknown>;
