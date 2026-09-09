async function target() {
  await import("./called.mts");
}

const invoke = target;
const calls = { run: invoke };
calls.run();
