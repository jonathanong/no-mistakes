export type QueryExecutor = <Row = unknown>(statement: {
  text: string;
}) => Promise<{ rows: Row[] }>;
export interface QueryOptions {
  client?: object;
  query?: QueryExecutor;
  readOnly?: boolean;
}
export declare function read(
  statement: string | { text: string },
  options?: QueryOptions,
): Promise<unknown>;
export declare function write(statement: { text: string }, options: QueryOptions): Promise<unknown>;
