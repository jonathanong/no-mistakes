export type TransactionQuery = <T = Record<string, unknown>>(sql: string, values: unknown[]) => Promise<{ rowCount: number | null; rows: T[] }>;
