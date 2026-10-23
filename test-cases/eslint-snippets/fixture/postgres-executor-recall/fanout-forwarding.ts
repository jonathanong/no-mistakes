import { query } from "@example/db";
Promise.all.call(Promise, items.map(query));
Promise[`all`][`apply`](Promise, [items.map(query)]);
Promise.all?.call?.(Promise, items.map(query));
(Promise.all as any)!.call(Promise, items.map(query));
Promise.all(items.map.call(items, query));
Promise.all(items[`map`]["apply"](items, [query]));
Promise.all.call(Promise, items.map.apply(items, [query]));
(0, Promise.all).call(Promise, items.map(query));
// The forwarded receiver supplies the collection, even if the method was read from an array.
Promise.all([].map.call(items, query));
Promise.all([].map.apply(items, [query]));
const all = Promise.all;
const alsoAll = all;
alsoAll.call(Promise, items.map(query));
const map = [].map;
Promise.all(map.call(items, query));
// Extra arguments do not hide the already evaluated mapped executor calls.
Promise.all(items.map(query), ...args);
Promise.all(items.map(query, ...args));
// Preserve the legacy conservative direct-map match; a tag can return "map".
Promise.all(items[tag`map`](query));
