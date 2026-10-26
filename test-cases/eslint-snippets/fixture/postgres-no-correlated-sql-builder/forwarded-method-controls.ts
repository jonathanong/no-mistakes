export function unknown(q, other, kind, values, key) {
  let mutable = q.append;
  const detached = q.append;
  if (kind === "a") q.append.apply(q, values);
  if (kind === "b") q.append.apply(q, [...values]);
  if (kind === "c") q.append.apply(q, [, "c"]);
  if (kind === "d") q.append.apply(q, []);
  if (kind === "e") q.append.apply(q, ["e", "extra"]);
  if (kind === "f") q.append.apply(null, ["f"]);
  if (kind === "g") q.append.apply(other.q, ["g"]);
  if (kind === "h") q.append.apply(q, ["h"], "extra");
  if (kind === "i") q.append.apply(q, ...values);
  if (kind === "j") mutable.call(q, "j");
  if (kind === "k") detached("k");
  if (kind === "l") q[key].apply(q, ["l"]);
  if (kind === "m") q.other.apply(q, ["m"]);
}
export function split(q, other, kind) {
  if (kind === "a") q.append.apply(q, ["a"]);
  if (kind === "b") q.append.apply(other, ["b"]);
}
export function reassigned(q, kind) {
  const append = q.append;
  append = other;
  if (kind === "a") append.call(q, "a");
  if (kind === "b") append.call(q, "b");
}
export function shadowed(q, kind) {
  const append = q.append;
  function inner(append) {
    if (kind === "a") append.call(q, "a");
    if (kind === "b") append.call(q, "b");
  }
}
// These bindings cannot establish a method reference from this file.
export function uncertain(q, kind, candidate) {
  declare const absent: unknown;
  const first = second;
  const second = first;
  const alias = candidate;
  const nothing = null;
  if (kind === "a") absent.call(q, "a");
  if (kind === "b") first.call(q, "b");
  if (kind === "c") alias.call(q, "c");
  if (kind === "d") nothing.call(q, "d");
  if (kind === "e") box.q.append.call(q, "e");
  if (kind === "f") q.append.apply(q, ["f", ...values]);
  if (kind === "g") q.append.apply(q, [undefined]);
  if (kind === "h") (0, q.append)("h");
}
export function sparseAndSpread(q, kind, values) {
  if (kind === "a") q.append.apply(q, [,]);
  if (kind === "b") q.append(...values);
}
export function changedForwarder(q, kind) {
  const add = (text) => q.append("literal");
  if (kind === "a") add("a");
  if (kind === "b") add("b");
}
