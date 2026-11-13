export type TransactionQuery = <T>(sql: string, values: unknown[]) => Promise<T>;
