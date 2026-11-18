import { read } from './db.mjs';

function execute(statement: string) {
  return read(statement);
}

function runSharedSearch(statement: string, callback: (statement: string) => Promise<void>) {
  return callback(statement);
}

export function listNamed() {
  return runSharedSearch('/* shared */ SELECT id FROM widgets LIMIT 5', execute);
}

export function listMissing() {
  return runSharedSearch('SELECT id FROM gadgets LIMIT 5', execute);
}
