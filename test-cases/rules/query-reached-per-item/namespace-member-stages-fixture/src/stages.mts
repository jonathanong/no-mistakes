declare function query(sql: string): Promise<unknown>;
export const STAGES = [{ run: () => query('SELECT 1') }];
