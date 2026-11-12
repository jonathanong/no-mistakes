import sql from 'sql-template-strings';
import { read } from '@bench/db';
import { addFilter, appendAlias } from './barrel.mjs';
import { appendNamespace } from './namespace.mjs';
import { typeFilter } from './type-only.mjs';
import { duplicateFilter } from './duplicate.mjs';
import { missingFilter } from './unresolved.mjs';
import { rewriteHead } from './helpers.mjs';

export function accepted() {
  const query = sql`/* accepted */ SELECT 1`;
  addFilter(query);
  read(query);
}
export function unsafe() {
  const query = sql`/* unsafe */ SELECT 1`;
  rewriteHead(query);
  read(query);
}
export function namedAlias() {
  const query = sql`/* alias */ SELECT 1`;
  appendAlias(query);
  read(query);
}
export function namespaceImport() {
  const query = sql`/* namespace */ SELECT 1`;
  appendNamespace(query);
  read(query);
}
export function typeOnlyImport() {
  const query = sql`/* type */ SELECT 1`;
  typeFilter(query);
  read(query);
}
export function duplicateImport() {
  const query = sql`/* duplicate */ SELECT 1`;
  duplicateFilter(query);
  read(query);
}
export function missingImport() {
  const query = sql`/* missing */ SELECT 1`;
  missingFilter(query);
  read(query);
}
