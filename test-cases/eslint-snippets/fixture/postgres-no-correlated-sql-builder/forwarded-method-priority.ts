export function configured(q, kind) {
  if (kind === "a") q.apply("a");
  if (kind === "b") q.apply("b");
}
export function configuredCall(q, kind) {
  if (kind === "a") q.call("a");
  if (kind === "b") q.call("b");
}
