export type Database = {
  query: (sql: string) => Promise<unknown>;
};
