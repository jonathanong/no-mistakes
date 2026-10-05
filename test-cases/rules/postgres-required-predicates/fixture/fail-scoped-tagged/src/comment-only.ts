import sql from "sql-template-strings";
import { query } from "@example/db";
export async function lockDocuments(ids: string[]) {
  return query(sql`/* lockDocuments */
    SELECT id
    FROM documents
    WHERE id = ANY(${ids}::uuid[])
  `)
}
