import { read, type QueryExecutor, type QueryOptions } from '@example/db';
declare function unconfigured(): any;
declare function opaque(value: unknown): unknown;
declare const dynamicSql: string;

// Return headers and unused produced values do not execute SQL.
const returnedDb = getDb(); // NON_SQL
returnedDb.query('SELECT 1 OFFSET 1'); // UNKNOWN_RETURN
function getDb(): QueryExecutor { // NON_SQL
  return unconfigured(); // NON_SQL
}
function getStore(): QueryOptions { // NON_SQL
  return unconfigured() ?? {}; // NON_SQL
}
const unusedStore = getStore(); // NON_SQL
const assertedDb = unconfigured() as QueryExecutor; // NON_SQL
assertedDb.query('SELECT 1 OFFSET 1'); // UNKNOWN_AS
const angleDb = <QueryExecutor>unconfigured(); // NON_SQL
angleDb.query('SELECT 1 OFFSET 1'); // UNKNOWN_ANGLE
const satisfiedDb = unconfigured() satisfies QueryExecutor; // NON_SQL
satisfiedDb.query('SELECT 1 OFFSET 1'); // UNKNOWN_SATISFIES
const arrowProducer = (): QueryExecutor => unconfigured(); // NON_SQL
const arrowDb = arrowProducer(); // NON_SQL
arrowDb('SELECT 1 OFFSET 1'); // UNKNOWN_ARROW
const producerAlias = getDb; // NON_SQL
const aliasDb = producerAlias(); // NON_SQL
aliasDb.query('SELECT 1 OFFSET 1'); // UNKNOWN_ALIAS
const receiverAlias = returnedDb; // NON_SQL
receiverAlias.query('SELECT 1 OFFSET 1'); // UNKNOWN_RECEIVER_ALIAS
getDb().query('SELECT 1 OFFSET 1'); // UNKNOWN_INLINE
opaque(getDb()); // UNKNOWN_OPAQUE
(unconfigured() as QueryExecutor).query('SELECT 1 OFFSET 1'); // UNKNOWN_INLINE_CAST

export function lexicalShadow() {
  function getDb() { return { query(_text: string) {} }; }
  const returnedDb = getDb(); // NON_SQL
  returnedDb.query('SELECT 1'); // NON_SQL: unrelated lexical declaration.
}
export function localType() {
  type QueryExecutor = (text: string) => void;
  const local = unconfigured() as QueryExecutor; // NON_SQL
  local('SELECT 1'); // NON_SQL: spelling is not imported type identity.
}

const objectDb = {
  db: unconfigured() as QueryExecutor, // NON_SQL
  run() { this.db.query('SELECT 1 OFFSET 1'); }, // UNKNOWN_OBJECT_METHOD
};
objectDb.db.query('SELECT 1 OFFSET 1'); // UNKNOWN_OBJECT_DIRECT
objectDb.run(); // NON_SQL: the executor site belongs to the method body.
objectDb.other.query('SELECT 1 OFFSET 1'); // NON_SQL: this field has no configured provenance.
const objectScopes = {
  db: unconfigured() as QueryExecutor, // NON_SQL
  run() {
    function dynamicThis() { this.db.query('SELECT 1'); } // NON_SQL
    const lexicalArrow = () => this.db.query('SELECT 1'); // UNKNOWN_OBJECT_ARROW
  },
  [this.db.query('SELECT 1')]() {}, // NON_SQL: keys evaluate with enclosing this.
};

export class CallbackCarrier {
  db!: QueryExecutor;
  run() {
    read('SELECT 1', { onRetry: this.db }); // UNKNOWN_FIELD_CALLBACK
  }
}

(null as unknown as QueryExecutor)('SELECT 1'); // UNKNOWN_NULL_CAST
({ query: unconfigured() } as QueryExecutor).query('SELECT 1'); // UNKNOWN_OBJECT_CAST
[unconfigured()] satisfies QueryExecutor; // NON_SQL: an unconsumed aggregate assertion.

// Existing indirect invocation operations remain unknown for possible fields.
export class IndirectCarrier {
  db!: QueryExecutor;
  run() {
    this.db.call(null, dynamicSql); // UNKNOWN_FIELD_CALL
    this.db['apply'](null, [dynamicSql]); // UNKNOWN_FIELD_APPLY
    this.db.bind(null, dynamicSql); // UNKNOWN_FIELD_BIND
    this.db.name.toString(); // NON_SQL: unrelated data method.
  }
}
objectDb.db['call'](null, dynamicSql); // UNKNOWN_OBJECT_CALL
objectDb.db.apply(null, [dynamicSql]); // UNKNOWN_OBJECT_APPLY
objectDb.db['bind'](null, dynamicSql); // UNKNOWN_OBJECT_BIND
objectDb.db.name.toString(); // NON_SQL: unrelated data method.
