async function target() {
  await import("./called.mts");
}

const invoke = target;
const calls = { run: invoke };

// Multi-hop: calls.run -> invoke -> target. One hop would stop at invoke.
export function callThroughObjectAlias() {
  calls.run();
}
