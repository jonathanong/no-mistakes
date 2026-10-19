/** Bound the complete SDK process, including Storybook/Vite cold optimization.
 * Native test and hook deadlines remain in the checked-in SDK configuration.
 * Plain Browser Mode is 15s: a cold Chromium launch under parallel CI workers
 * can pass 5s before Vitest prints more than its banner, and the helper then
 * SIGKILLs the child (Node reports that as a null exit code). Storybook stays
 * at 20s for Vite. Both leave the 2s close and group-drain barriers inside
 * the 30s parent suite timeout.
 */
export function browserChildContainment(storybook) {
  return { watchdogMs: storybook ? 20000 : 15000 };
}
