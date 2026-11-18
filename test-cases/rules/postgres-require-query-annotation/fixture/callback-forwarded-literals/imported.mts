import { read } from './db.mjs';
import { runImportedSearch } from './helper.mjs';

export function listImportedWidgets() {
  return runImportedSearch('/* listImportedWidgets */ SELECT id FROM widgets LIMIT 5', statement => read(statement));
}
