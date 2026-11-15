export interface PoolClient {
  query(sql: string, binds: unknown[]): Promise<unknown>;
  release(): void;
}
export interface Pool {
  query(sql: string, binds: unknown[]): Promise<unknown>;
  end(): Promise<void>;
}
export type Execute = (sql: string, binds: unknown[]) => Promise<unknown>;
