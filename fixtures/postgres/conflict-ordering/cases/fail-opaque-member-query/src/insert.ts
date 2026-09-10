declare function assembleWriter(): string;
declare const client: { query(sql: string): unknown };

client.query(assembleWriter());
