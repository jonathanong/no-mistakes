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

export function localAnnotatedStatement() { return sql`/* imported local tag */ SELECT 1`; }
export function localUnannotatedStatement() { return sql`SELECT 1`; }

function preservedVarParameter(statement: unknown) {
  var statement;
  write(statement); // known:bare-var-parameter
}
preservedVarParameter(sql`/* parameter preserved */ SELECT 1`);
function preservedVarFunction() { return sql`/* function preserved */ SELECT 1`; }
var preservedVarFunction;
write(preservedVarFunction()); // known:bare-var-function
var unboundVar;
write(unboundVar); // unanalyzable:bare-var-unbound

// Uncalled speculative functions must not change another entrypoint's state.
const firstSharedBuilder = sql`/* isolated first */ SELECT 1`;
const secondSharedBuilder = sql`/* isolated second */ SELECT 1`;
export function aUnusedMutation() { unknownMutation(firstSharedBuilder); }
export function zIsolatedExecution() {
  write(firstSharedBuilder); // known:isolated-late-entrypoint
}
export function aIsolatedExecution() {
  write(secondSharedBuilder); // known:isolated-early-entrypoint
}
export function zUnusedMutation() { unknownMutation(secondSharedBuilder); }

const primitiveCapture = '/* primitive capture */ SELECT 1';
function unsupportedLocalBindings() {
  const local = 'SELECT 2';
  function inner() { return local; }
  var temporary;
  if (flag) return inner();
}
export function unsupportedCapturedPrimitive() {
  unsupportedLocalBindings();
  write(primitiveCapture); // unanalyzable:unsupported-captured-primitive
}
function unsupportedPrimitiveShadow() {
  var primitiveCapture;
  if (flag) return primitiveCapture;
}
export function unsupportedVarShadowPreservesPrimitive() {
  unsupportedPrimitiveShadow();
  write(primitiveCapture); // known:unsupported-var-shadow
}
