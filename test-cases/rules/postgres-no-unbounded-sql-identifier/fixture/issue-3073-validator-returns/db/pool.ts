export type Query = (sql: string, values?: unknown[]) => Promise<unknown>;
export declare const query: Query;
export declare const read: Query;
export declare const write: Query;
