import sql from "sql-template-strings";

export const imported = sql`SELECT * FROM ${process.env.IMPORTED_TABLE}`;
