function target() {}

// Duplicate keys keep last-write. `run: 0` overwrites the callable.
const calls = { run: target, run: 0 };

export function callDuplicateKey() {
  calls.run();
}
