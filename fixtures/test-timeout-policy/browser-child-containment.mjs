/** Bound the complete SDK process, including Storybook/Vite cold optimization.
 * Native test and hook deadlines remain in the checked-in SDK configuration.
 */
export function browserChildContainment(storybook) {
  return { watchdogMs: storybook ? 20000 : 5000 };
}
