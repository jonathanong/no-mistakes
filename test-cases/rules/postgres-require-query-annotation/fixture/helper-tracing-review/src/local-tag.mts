import { write } from '@app/db';
function sql(strings: TemplateStringsArray, ...values: unknown[]) { return strings.join('?'); }
export function localTag(id: string) {
  return write(sql`/* legacy local */ SELECT ${id}`); // known:local-tag
}
export function parameterTag(sql: (strings: TemplateStringsArray) => unknown) {
  return write(sql`/* parameter tag */ SELECT 1`); // unanalyzable:parameter-tag
}
export function nestedTag() {
  function sql() { return 'SELECT 1'; }
  return write(sql`/* arbitrary helper */ SELECT 1`); // unanalyzable:nested-tag
}

export function differentLocalTagImplementation() {
  function sql(strings: TemplateStringsArray) { return 'SELECT 2'; }
  write(sql`/* not proven */ SELECT 1`); // unanalyzable:different-local-tag
}

// The original module tag retains legacy facts when invoked before any shadow.
write(sql`/* module local tag */ SELECT 1`); // known:module-local-tag
