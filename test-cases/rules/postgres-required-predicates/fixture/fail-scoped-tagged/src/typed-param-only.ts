import sql from "sql-template-strings";
import type { TransactionQuery } from "@example/db/types";
export async function lockDocuments(query: TransactionQuery, ids: string[]) {
  return query(sql`
    SELECT id
    FROM documents
    WHERE id = ANY(${ids}::uuid[])
  `)
}
