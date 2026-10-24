import SQL from 'sql-template-strings';
import { read } from '@example/db';
function boundTable(value: string) { return SQL`SELECT * FROM tenant_${value}`; }
function quotedTable(value: string) { return SQL`SELECT * FROM "${value}"`; }
function authoredTable() { return SQL`SELECT * FROM "tenant_$1_sql_placeholder_1"`; }
export function placements(value: string) {
  read(boundTable(value)); // UNKNOWN_EMBEDDED_BIND
  read(quotedTable(value)); // UNKNOWN_QUOTED_BIND
  read(authoredTable()); // CLEAR_AUTHORED_PLACEHOLDER_TEXT
}
