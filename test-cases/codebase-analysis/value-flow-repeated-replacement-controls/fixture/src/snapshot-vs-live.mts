import { snapshotDest, liveDest, source, rhs } from "@vendor/acquisition-controls";
// This method is copied once; later mutations of source do not change it.
const savedRhs = source.get;
for (const item of ["a", "b"]) {
  snapshotDest.get(item);
  liveDest.get(item);
  snapshotDest.get = savedRhs;
  // A live RHS acquired in this loop cannot inherit source's next value.
  liveDest.get = source.get;
  source.get = rhs.get;
}
