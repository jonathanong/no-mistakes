import { forbidden as boundary } from './dependency';
export function ambientBridge() { boundary(); }
// Parser-accepted ambient imports historically overwrite the file-wide table,
// even though this declaration isn't part of the helper's lexical scope.
declare module 'ambient-provider' {
  import { safe as boundary } from './safe';
}
