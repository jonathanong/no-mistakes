import type { PoolClient as Client } from '@example/db';

class StaticOnly {
  static db: Client;
  static run() { this.db.query('static receiver is not an instance'); }
}
class UntypedProperty {
  // A parameter property without an imported annotation has no type provenance.
  constructor(private db) {}
  run() { this.db.query('untyped property'); }
}
function unknownShapes(array: Array, context: { missing; [dynamicKey]: Client; method(): void }) {
  array[0].query('Array without an element type');
  context.missing.query('untyped property');
  getContext().db.query('factory receiver has no declared binding');
  getContexts()[0].query('computed factory receiver');
  unknownContext.db.query('unbound root');
  this.db.query('no class receiver');
}
function redeclaration(client: Client) {
  // A var redeclaration owns the existing explicitly typed parameter binding.
  var client = unrelated();
  client.query('SELECT 1'); // diagnostic
}
