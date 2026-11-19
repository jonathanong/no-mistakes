export type QueryExecutor = <Row = unknown>(statement: {
  text: string;
}) => Promise<{ rows: Row[] }>;
