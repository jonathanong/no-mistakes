import sql from "sql-template-strings";
import type { SQLStatement } from "sql-template-strings";

export type RelationConfig = { relationTable: string; idColumn: string; cteAlias: string };

// The option properties are identifiers, appended as raw text: they only
// recover when the call site writes each one as a string literal.
export function buildFollowedCte(userId: string, config: RelationConfig): SQLStatement {
  return sql`/* fragment */ `
    .append(config.cteAlias)
    .append(sql` AS (SELECT object_id AS `)
    .append(config.idColumn)
    .append(sql` FROM `)
    .append(config.relationTable)
    .append(sql` WHERE subject_id = ${userId})`);
}
