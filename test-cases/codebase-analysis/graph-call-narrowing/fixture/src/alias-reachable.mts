async function target() {
  await import("./called.mts");
}

const invoke = target;
invoke();
