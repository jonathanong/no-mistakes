// Newly recognized receivers must not displace the existing builder in a guard.
export function applied(q, other, kind) {
  if (kind === "a") q.append("a");
  if (kind === "b") {
    other.append.apply(other, ["x"]);
    q.append("b");
  }
}
export function aliased(q, other, kind) {
  const append = other.append;
  if (kind === "a") q.append("a");
  if (kind === "b") {
    append.apply(other, ["x"]);
    q.append("b");
  }
}
export function both(q, other, kind) {
  if (kind === "a") {
    other.append.apply(other, ["x"]);
    q.append("a");
  }
  if (kind === "b") {
    q.append("b");
    other.append.call(other, "y");
  }
}
export function oneGuard(q, kind) {
  // Two appends inside a single guard are not repeated independent guards.
  if (kind === "a") {
    q.append.apply(q, ["a"]);
    q.append("b");
  }
}
