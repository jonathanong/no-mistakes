import sql from 'sql-template-strings';
import { read } from './db.mjs';

type Run = typeof read;

export function lookup(opts: { query?: Run }, item: number) {
  const run = opts.query ?? read;
  return run(sql`/* lookup */ SELECT id FROM widgets WHERE id = ${item}`);
}

function runSearch(statement: string, execute: (statement: string) => Promise<void>) {
  return execute(statement);
}

export function listWidgets() {
  return runSearch('/* listWidgets */ SELECT id FROM widgets LIMIT 5', statement => read(statement));
}

export function listGadgets() {
  return runSearch('/* listGadgets */ SELECT id FROM gadgets LIMIT 5', statement => read(statement));
}

export function listWithoutAnnotation() {
  return runSearch('SELECT id FROM widgets LIMIT 5', statement => read(statement));
}

export function listWithDynamicName(name: string) {
  return runSearch(`/* ${name} */ SELECT id FROM widgets LIMIT 5`, statement => read(statement));
}

export function listWithDynamicPrefix(prefix: string) {
  return runSearch(`${prefix}SELECT id FROM widgets LIMIT 5`, statement => read(statement));
}

export function lookupThroughCallback(item: number) {
  return runSearch(`/* lookupThroughCallback */ SELECT id FROM widgets WHERE id = ${item}`, statement => read(statement));
}

export function overwriteInCallback() {
  return runSearch('/* overwritten */ SELECT id FROM widgets LIMIT 5', statement => {
    statement = 'SELECT id FROM gadgets LIMIT 5';
    return read(statement);
  });
}
