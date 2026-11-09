import { forbidden as boundary } from './dependency';
export function globalBridge() { boundary(); }
// These imports historically update the file-wide table.
declare global {
  import { safe as boundary } from './safe';
}
