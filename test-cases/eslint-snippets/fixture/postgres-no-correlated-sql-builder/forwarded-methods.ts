// Explicit receivers keep detached methods attributable to the guarded builder.
export function applied(q, kind) {
  if (kind === "a") q.append.apply(q, ["a"]);
  if (kind === "b") q[`append`]["apply"](q, ["b"]);
}
export function aliased(q, kind) {
  const append = q.append;
  const again = append;
  if (kind === "a") again.call(q, "a");
  if (kind === "b") append.apply(q, ["b"]);
}
export function comma(q, kind) {
  if (kind === "a") (0, q.append).call(q, "a");
  if (kind === "b") (0, q.append).apply(q, ["b"]);
}
export function optional(q, kind) {
  if (kind === "a") (q.append as any)!.apply?.(q, ["a"]);
  if (kind === "b") q.append?.apply?.(q, ["b"]);
}
export function otherReceiver(q, other, kind) {
  if (kind === "a") q.append.apply(other, ["a"]);
  if (kind === "b") q.append.call(other, "b");
}
