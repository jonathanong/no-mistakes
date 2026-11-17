import { read, type QueryExecutor } from '@example/db';

function dynamicKey(key: string) {
  const helpers = { load(query: QueryExecutor) { return query('SELECT id FROM t'); } };
  helpers[key](read);
}
function getter() {
  const helpers = { get load() { return (query: QueryExecutor) => query('SELECT id FROM t'); } };
  helpers.load(read);
}
function mutableReceiver() {
  let helpers = { load(query: QueryExecutor) { return query('SELECT id FROM t'); } };
  helpers.load(read);
}
function reassignedReceiver() {
  let helpers = { load(query: QueryExecutor) { return query('SELECT id FROM t'); } };
  helpers = { load(query: QueryExecutor) { return query('SELECT id FROM other'); } };
  helpers.load(read);
}
function replacedProperty() {
  const helpers = { load(query: QueryExecutor) { return query('SELECT id FROM t'); } };
  helpers.load = (query: QueryExecutor) => query('SELECT id FROM other');
  helpers.load(read);
}
function aliasMutation() {
  const helpers = { load(query: QueryExecutor) { return query('SELECT id FROM t'); } };
  const alias = helpers;
  alias.load = (query: QueryExecutor) => query('SELECT id FROM other');
  helpers.load(read);
}
function escapedReceiver(touch: (value: unknown) => void) {
  const helpers = { load(query: QueryExecutor) { return query('SELECT id FROM t'); } };
  touch(helpers);
  helpers.load(read);
}
function unknownProperty() {
  const helpers = { load(query: QueryExecutor) { return query('SELECT id FROM t'); } };
  helpers.missing(read);
}
function spreadReceiver(other: object) {
  const helpers = { ...other, load(query: QueryExecutor) { return query('SELECT id FROM t'); } };
  helpers.load(read);
}
function duplicateProperty() {
  const helpers = {
    load(query: QueryExecutor) { return query('SELECT id FROM t'); },
    load(query: QueryExecutor) { return query('SELECT id FROM other'); },
  };
  helpers.load(read);
}
function optionalReceiver() {
  const helpers = { load(query: QueryExecutor) { return query('SELECT id FROM t'); } };
  helpers?.load(read);
}
function optionalCall() {
  const helpers = { load(query: QueryExecutor) { return query('SELECT id FROM t'); } };
  helpers.load?.(read);
}
function usesReceiver() {
  const helpers = { table: 't', load(query: QueryExecutor) { return query(this.table); } };
  helpers.load(read);
}
function shadowedReceiver() {
  const helpers = { load(query: QueryExecutor) { return query('SELECT id FROM t'); } };
  return function(helpers: { load: (query: QueryExecutor) => unknown }) {
    helpers.load(read);
  };
}
function nestedReceiver(touch: (value: unknown) => void) {
  const helpers = { load(query: QueryExecutor) {
    class Nested extends (touch(this), Object) {}
    return query('SELECT id FROM t');
  } };
  helpers.load(read);
}
function prototypeReceiver(parent: object) {
  const helpers = { __proto__: parent, load(query: QueryExecutor) { return query('SELECT id FROM t'); } };
  helpers.load(read);
}
function computedInitializer(key: string) {
  const helpers = { [key](query: QueryExecutor) { return query('SELECT id FROM t'); } };
  helpers.load(read);
}
