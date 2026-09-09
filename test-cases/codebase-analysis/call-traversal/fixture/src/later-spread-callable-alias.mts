function target() {}

function replacement() {
  return {};
}
// A later unknown spread may overwrite `run`, so the member alias must not survive.
const calls = { run: target, ...replacement() };

export function callAfterSpread() {
  calls.run();
}
