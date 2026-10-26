import { read } from '@example/db';
import type { QueryOptions, QueryExecutor } from '@example/db';
import SQL from 'sql-template-strings';
declare const statement: string;
declare const value: unknown;
export class Receiver {
  options!: QueryOptions;
  bind() {
    read(SQL`SELECT ${this.options.query} AS value`); // CLEAR: field is bound data only.
  }
  metadata() {
    read('SELECT 1', this.options); // CLEAR: field is metadata only.
  }
  execute() {
    this.options.query(statement); // UNKNOWN: actual dynamic SQL call stays uncertain.
  }
}
export function erasedBind() {
  read(SQL`SELECT ${value as QueryExecutor} AS value`); // CLEAR: assertion is erased inside a bind.
}
export function erasedDynamic(query: QueryExecutor) {
  read(SQL`SELECT ${(query as QueryExecutor)(statement)} AS value`); // UNKNOWN: actual call still executes.
}
declare const opaque: (value: unknown) => unknown;
export class OtherFields {
  #privateOptions!: QueryOptions;
  ['computedOptions']!: QueryOptions;
  bind() {
    read(SQL`SELECT ${this.#privateOptions.query} AS value`); // CLEAR: private field data span.
    read(SQL`SELECT ${this['computedOptions'].query} AS value`); // CLEAR: computed field data span.
  }
  escaped() {
    read(SQL`SELECT ${opaque(this.#privateOptions)} AS value`); // UNKNOWN: opaque member use.
  }
}
