import * as db from "@example/db";
declare function invoke(executor: typeof db.read): void;

export function inspect() {
  invoke(db.read);
  const { read } = db;
  invoke(read);
  const alias = db as typeof db;
  invoke(alias.read);
  const { ...rest } = db;
  invoke(rest.read);
  const key = "read" as const;
  invoke(db[key]);
  const {
    nested: { read: nestedRead },
  } = { nested: db };
  invoke(nestedRead);
}

export function additionalNamespaceForms() {
  const key = "read" as const;
  const { [key]: computedDestructured } = db;
  invoke(computedDestructured);
  const { write: _write, ...remaining } = db;
  invoke(remaining.read);
  const copy = { ...db };
  invoke(copy.read);
  const {
    nested: { ...nestedRest },
  } = { nested: db as typeof db };
  invoke(nestedRest.read);
  const holder = { inner: db };
  const {
    inner: { read: fromHolder },
  } = holder;
  invoke(fromHolder);
  const { read: fromSpread } = { ...db };
  invoke(fromSpread);
  const {
    inner: { read: fromSpreadNested },
  } = { ...holder };
  invoke(fromSpreadNested);
}
