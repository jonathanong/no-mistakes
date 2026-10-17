import { read } from '@example/db';
export function opaqueBeforeAllocation(mutator: () => unknown) {
  mutator();
  read(['SELECT 1'].join(''));
}

export function falseBranch(mutator: () => unknown) {
  if (false) mutator();
  read(['SELECT 1'].join(''));
}
export function conditionalEffect(flag: boolean, mutator: () => unknown) {
  if (flag) mutator();
  read(['SELECT 1'].join(''));
}
export function pristineAfterOpaque() {
  read(['SELECT 1'].join(''));
}
export function computedBeforeOpaque(mutator: () => unknown) {
  const text = ['SELECT 1'].join('');
  mutator();
  read(text);
}
function build() { return ['SELECT 1'].join(''); }
export function reusedCallableContext(mutator: () => unknown) {
  read(build());
  mutator();
  // Reusing the callable must not retain its earlier intrinsic trust.
  read(build());
}
