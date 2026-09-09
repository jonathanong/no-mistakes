async function target() {
  await import("./called.mts");
}

const invoke = target;
// Increment invalidates the alias, so `target`'s import stays unreachable.
invoke++;
invoke();
