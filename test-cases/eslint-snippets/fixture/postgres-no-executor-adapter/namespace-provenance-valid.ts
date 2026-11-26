import * as db from "@example/db";
declare function invoke(executor: unknown): void;

export function excludedAndOverridden(
  dynamicKey: string,
  fallback: unknown,
  unknownHolder: { nested: typeof db },
  getHolder: () => { nested: typeof db },
) {
  const { read: _stripped, ...withoutRead } = db;
  invoke(withoutRead.read);
  const { read: defaulted = fallback, ...restAfterDefault } = db;
  invoke(defaulted);
  void restAfterDefault;
  const overridden = { ...db, read: fallback };
  invoke(overridden.read);
  const withGetter = {
    get read() {
      return fallback;
    },
  };
  invoke(withGetter.read);
  const [fromTuple] = [db];
  invoke(fromTuple.read);
  const {
    nested: { read: fromUnknownParam },
  } = unknownHolder;
  invoke(fromUnknownParam);
  const {
    nested: { read: fromCallResult },
  } = getHolder();
  invoke(fromCallResult);
  const dynamicOverride = { ...db, [dynamicKey]: fallback };
  invoke(dynamicOverride.read);
  const {
    nested: { read: withDefault = fallback },
  } = { nested: db };
  invoke(withDefault);
  let mutableKey = "read";
  invoke(db[mutableKey]);
  invoke(db[dynamicKey]);
  let mutableAlias = db;
  invoke(mutableAlias.read);
  const cycleA: typeof db = cycleB;
  const cycleB: typeof db = cycleA;
  invoke(cycleA.read);
  const {
    missing: { read: missingNested },
  } = { other: db } as unknown as { missing: typeof db };
  invoke(missingNested);
}
