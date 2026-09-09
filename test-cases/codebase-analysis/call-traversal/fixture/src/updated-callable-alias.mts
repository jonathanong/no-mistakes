function target() {}

const alias = target;
// Increment is a write: the later call must not keep a call edge to `target`.
alias++;

export function callAfterUpdate() {
  alias();
}
