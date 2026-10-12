import { read as load } from '@data-stores/psql';
import { valkey } from '@data-stores/valkey';
export function lookup(item: unknown) { return inner(item); }
function inner(item: unknown) { return load(item); }
export function cachedLookup(item: unknown) { return valkey.hget(item); }
// Transitive reachability must not turn unrelated same-name calls into sinks.
export function unrelated(item: unknown) { return stream(item); }
function stream(item: unknown) { return console.write(item); }
