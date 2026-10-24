import { collection as importedCollection, query, type QueryExecutor } from "@example/db";

const map = new Map<QueryExecutor, unknown>();
map.get(query);
map.set(query, "key");
map.has(query);
map.delete(query);
map.set("key", query);

const weakMap = new WeakMap<QueryExecutor, unknown>();
weakMap.get(query);
weakMap.set(query, "key");
weakMap.has(query);
weakMap.delete(query);
weakMap.set({}, query);

const set = new Set<QueryExecutor>();
set.has(query);
set.delete(query);
set.add(query);

const weakSet = new WeakSet<QueryExecutor>();
weakSet.has(query);
weakSet.delete(query);
weakSet.add(query);

const custom = {
  get(key: QueryExecutor) {
    return key;
  },
};
custom.get(query);

declare const uncertain: unknown;
(uncertain as Map<QueryExecutor, unknown>).get(query);
declare const union: Map<QueryExecutor, unknown> | Set<QueryExecutor>;
union.has(query);

class LookalikeMap<K, V> {
  get(_key: K): V | undefined {
    return undefined;
  }
}
const lookalike = new LookalikeMap<QueryExecutor, unknown>();
lookalike.get(query);

function shadowedBuiltin() {
  class Map<K, V> {
    get(_key: K): V | undefined {
      return undefined;
    }
  }
  const fake = new Map<QueryExecutor, unknown>();
  fake.get(query);
}

class OverridingMap<K, V> extends Map<K, V> {
  override get(key: K): V | undefined {
    return undefined;
  }
}
const overriding = new OverridingMap<QueryExecutor, unknown>();
overriding.get(query);

map["get"](query);
map.get.call(map, query);
map.get(...[query]);
new Map<QueryExecutor, unknown>().get(query);

declare const untyped: any;
untyped.get(query);

function receive(_value: unknown) {}
receive([query]);
receive((sql: string) => query(sql));

function parameter(collection: Map<QueryExecutor, unknown>) {
  collection.get(query);
}
let reassigned = new Map<QueryExecutor, unknown>();
reassigned = new Map();
reassigned.get(query);
missingCollection.get(query);

const mapAlias = map;
mapAlias.get(query);
const maskedSubclass: Map<QueryExecutor, unknown> = new OverridingMap();
maskedSubclass.get(query);
const assertedSubclass = new OverridingMap<QueryExecutor, unknown>() as unknown as Map<
  QueryExecutor,
  unknown
>;
assertedSubclass.get(query);
function mapFactory(): Map<QueryExecutor, unknown> {
  return new Map();
}
const factoryMap = mapFactory();
factoryMap.get(query);
importedCollection.get(query);
declare const unavailableMap: Map<QueryExecutor, unknown>;
unavailableMap.get(query);
const importedAlias = importedCollection;
importedAlias.get(query);
map?.get(query);
