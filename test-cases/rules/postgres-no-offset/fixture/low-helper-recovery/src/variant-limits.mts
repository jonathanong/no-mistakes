import SQL from 'sql-template-strings';
import { read } from '@example/db';

function selectVariant(enabled: boolean) {
  // Keep the known violation first so a one-variant budget retains its evidence.
  if (enabled) return SQL`SELECT 'private-sql-payload' OFFSET 8`;
  return SQL`SELECT 1`;
}

export function run(enabled: boolean) {
  read(selectVariant(enabled));
}
