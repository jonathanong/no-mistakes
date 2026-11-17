import sql from "sql-template-strings";

export const namespaceStatement = sql`SELECT * FROM ${process.env.NAMESPACE_TABLE}`;
