export function suppressed(q, kind) {
  // no-mistakes-disable-next-line postgres-no-correlated-sql-builder
  if (kind === "a") q.append.apply(q, ["a"]);
  if (kind === "b") q.append.apply(q, ["b"]);
}
export function lineSuppressed(q, kind) {
  if (kind === "a") q.append.apply(q, ["a"]); // no-mistakes-disable-line postgres-no-correlated-sql-builder
  if (kind === "b") q.append.apply(q, ["b"]);
}
