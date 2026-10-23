// no-mistakes-disable-file postgres-no-unbounded-query-fanout -- bounded upstream
import { query } from "@example/db";
Promise.all.call(Promise, items.map.call(items, query));
Promise.all.apply(Promise, [items.map.apply(items, [query])]);
