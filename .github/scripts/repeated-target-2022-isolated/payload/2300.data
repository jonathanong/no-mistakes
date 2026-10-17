import { primary, session, replica, scoped, local, standalone, nested } from '@vendor/valkey';
function replacement() {}
function client() { return primary; }
client().get('key');
primary.get = replacement;
function clientAfter() { return session; }
session.get = replacement;
clientAfter().get('key');
function copied() { return replica.get; }
const saved = copied();
replica.get = replacement;
saved('key');
function aliasCopy() { const get = scoped.get; return get; }
const aliasSaved = aliasCopy();
scoped.get = replacement;
aliasSaved('key');
function localCopy() { const get = local.get; local.get = replacement; return get; }
const localSaved = localCopy();
localSaved('key');
const outside = standalone.get;
function outsideCopy() { return outside; }
const outsideSaved = outsideCopy();
standalone.get = replacement;
outsideSaved('key');
function nestedCopy() {
  const get = nested.get;
  function innerCopy() { return get; }
  nested.get = replacement;
  return innerCopy();
}
const nestedSaved = nestedCopy();
nestedSaved('key');
