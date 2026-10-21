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
  // An unknown callee must still expose its nested require inside this scope.
  getHandler()(require('./scoped-unknown-callee-arg.js'));
  return require.resolve('./resolved.js');
}

// These graph call sites still contain imports in their child expressions.
[1].map(helper);
[1].map(externalCallback);
new Handler(require('./constructor-arg.js'));
tag`${require('./tag-arg.js')}`;
consume?.(require('./optional-argument.js'));
getHandler()(require('./unknown-callee-arg.js'));

function shadowed(require: (specifier: string) => unknown) {
  // A lexical `require` is a call, but not a CommonJS import.
  return require('./shadowed.js');
}

export function run(_account: Account) {
  helper();
  return require('./required.js');
}
