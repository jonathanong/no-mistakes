import type { Account } from './types.js';
import './startup.js';
export { value } from './exported.js';

const load = () => import('./lazy.js');
export const feature = makeFeature(() => import('./reachable.js'));
export const registry = { handler: () => import('./registry.js') };

function helper() {
  // Keep a nested runtime import and a callable alias: full facts finalize
  // these relationships, while production dependency checks only need syntax.
  const alias = load;
  alias();
  return require.resolve('./resolved.js');
}

export function run(_account: Account) {
  helper();
  return require('./required.js');
}
