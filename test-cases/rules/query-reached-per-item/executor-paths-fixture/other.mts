export type TransactionQuery = (sql: string) => Promise<unknown>
export type QueryOptions = { query?: TransactionQuery }
