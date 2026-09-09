const calls = { run: missing };

// Unresolved last hop stays conservative; no local callable edge.
export function callUnresolvedAlias() {
  calls.run();
}
