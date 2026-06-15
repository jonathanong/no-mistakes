// Named re-export barrel. It forwards `used` but never calls it; the unrelated
// local `used()` below must NOT be reported as a call site of mod#used.
export { used } from "./mod";

function unrelated() {
  const used = () => 0;
  return used();
}

export const keepUnrelated = unrelated;
