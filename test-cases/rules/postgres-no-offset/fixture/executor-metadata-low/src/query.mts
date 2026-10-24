import { read } from '@example/db';
import type { QueryOptions, QueryExecutor } from '@example/db';
declare const unknownHelper: (value: unknown) => void;
export function literal(options: QueryOptions) {
  read('SELECT 1', options); // SQL is known; the options argument is metadata.
}
export function dynamic(options: QueryOptions, statement: string) {
  read(statement, options); // Keep exactly the dynamic statement obligation.
}
export function escaped(options: QueryOptions) {
  unknownHelper(options); // Unknown helper transport must not disappear.
}
export function aliased(query: QueryExecutor, statement: string) {
  const alias = query;
  alias(statement); // A real unknown executor call must still advise.
}
export function destructured(options: QueryOptions, statement: string) {
  const { query } = options;
  query(statement); // Receiver destructuring needs actual-call negative tracking.
}
export function nested(options: QueryOptions) {
  read('SELECT 1', unknownHelper(options)); // ADVISE: nested opaque call owns this use.
  read('SELECT 1', () => options); // ADVISE: callback body is an escape.
  read('SELECT 1', function () { return options; }); // ADVISE: function body is an escape.
  read('SELECT 1', new Opaque(options)); // ADVISE: opaque constructor owns this use.
  read('SELECT 1', opaqueTag`${options}`); // ADVISE: opaque tag owns this use.
}
declare const Opaque: new (value: unknown) => unknown;
declare const opaqueTag: (strings: TemplateStringsArray, ...args: unknown[]) => unknown;
