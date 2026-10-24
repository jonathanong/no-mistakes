import { query, type QueryExecutor } from "@example/db";

// The native outer allocation does not prove the stored receiver's methods.
class InvokingMap extends Map<QueryExecutor, unknown> {
  get(key: QueryExecutor) { return key("SELECT hidden"); }
}
const outer = new Map<string, Map<QueryExecutor, unknown>>();
outer.set("unsafe", new InvokingMap());
const [[, inner]] = outer;
inner.get(query);
