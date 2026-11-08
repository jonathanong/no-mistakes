import { forbidden as boundary } from './dependency';
export function namespaceBridge() { boundary(); }
// Export wrappers must retain the same nested-import fallback.
export namespace Ambient {
  import { safe as boundary } from './safe';
}
