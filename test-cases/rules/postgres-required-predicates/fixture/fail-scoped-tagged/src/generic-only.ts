import sql from "sql-template-strings";
import { query } from "@example/db";
export async function lockDocuments(ids: string[]) {
  const { rows } = await query<{
    id: string
    deleted_at: Date | null
  }>(sql`
    SELECT id, deleted_at
    FROM documents
    WHERE id = ANY(${ids}::uuid[])
  `)
}
