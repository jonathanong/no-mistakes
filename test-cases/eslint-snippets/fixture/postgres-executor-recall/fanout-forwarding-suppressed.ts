import { query } from "@example/db";
// no-mistakes-disable-next-line postgres-no-unbounded-query-fanout -- bounded upstream
Promise.all.call(Promise, items.map.call(items, query));
Promise.all.apply(Promise, [items.map.apply(items, [query])]); // no-mistakes-disable-line postgres-no-unbounded-query-fanout -- bounded upstream
