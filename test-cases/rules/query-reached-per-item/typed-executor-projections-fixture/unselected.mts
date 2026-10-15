// The same exported spelling from another module is not a selected executor.
export type TransactionQuery = (sql: string) => Promise<unknown>;
export type QueryOptions = { query?: TransactionQuery };
export declare function write(sql: string): Promise<unknown>;
export type NonNullable<T> = (sql: string) => Promise<unknown>;
