// no-mistakes-disable-file postgres-no-correlated-sql-builder
export function suppressed(q, kind) {
  if (kind === "a") q.append.apply(q, ["a"]);
  if (kind === "b") q.append.apply(q, ["b"]);
}
