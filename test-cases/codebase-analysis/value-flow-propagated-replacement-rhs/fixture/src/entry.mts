import {
  source,
  dest,
  unrelatedSource,
  unrelatedDest,
  snapshotSource,
  snapshotDest,
  functionSource,
  functionDest,
} from "@vendor/propagated-replacement-rhs";

function replacement() {}
const box = { client: source };
box.client.get = replacement;
for (const item of ["a", "b"]) {
  dest.get(item);
  dest.get = source.get;
  const after = dest.get;
  after(item);
}
dest.set("outside", "value");

// A different propagated member does not replace the RHS method.
const unrelated = { client: unrelatedSource };
unrelated.client.set = replacement;
for (const item of ["a", "b"]) {
  unrelatedDest.get(item);
  unrelatedDest.get = unrelatedSource.get;
}

// This module copy predates the aliased write and remains an original snapshot.
const savedModule = snapshotSource.get;
const snapshotBox = { client: snapshotSource };
snapshotBox.client.get = replacement;
for (const item of ["a", "b"]) {
  snapshotDest.get(item);
  snapshotDest.get = savedModule;
}

function copyOnce() {
  // Its one closed module invocation acquires the copy before this alias write.
  const savedFunction = functionSource.get;
  const functionBox = { client: functionSource };
  functionBox.client.get = replacement;
  for (const item of ["a", "b"]) {
    functionDest.get(item);
    functionDest.get = savedFunction;
  }
}
copyOnce();
