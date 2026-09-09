function target() {}
function later() {}

// Duplicate keys keep last-write, so `run` still aliases `later`.
const calls = { run: target, run: later };

export function callLastWrite() {
  calls.run();
}
