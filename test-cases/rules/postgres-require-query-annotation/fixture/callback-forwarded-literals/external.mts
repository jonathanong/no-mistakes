import { read } from './db.mjs';

export function execute(statement: string) {
  return read(statement);
}

function runExternalSearch(statement: string, callback: (statement: string) => Promise<void>) {
  return callback(statement);
}

runExternalSearch('/* locallyKnown */ SELECT id FROM widgets LIMIT 5', execute);
