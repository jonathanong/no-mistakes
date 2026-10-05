import sql from "sql-template-strings";
import type { TransactionQuery } from "@example/db/types";
export async function lockDocuments(query: TransactionQuery, ids: string[]) {
  const { rows } = await query<{
    id: string
    deleted_at: Date | null
  }>(sql`/* lockDocuments */
    SELECT id, deleted_at
    -- no-mistakes-disable-next-line postgres-required-predicates
    FROM documents
    WHERE id = ANY(${ids}::uuid[])
    ORDER BY id
    FOR KEY SHARE
  `)
}
