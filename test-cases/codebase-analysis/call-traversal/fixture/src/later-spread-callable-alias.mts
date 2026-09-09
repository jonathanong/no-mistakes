function target() {}

const replacement = {};
// A later spread may overwrite `run`, so the member alias must not survive.
const calls = { run: target, ...replacement };

export function callAfterSpread() {
  calls.run();
}
