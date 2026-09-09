function target() {}

const wrapper = { run: invoke };
const invoke = wrapper.run;

// Cycle: wrapper.run <-> invoke. Stay conservative; no call edge to `target`.
export function callCyclicAlias() {
  wrapper.run();
}
