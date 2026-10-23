import { forbidden as boundary } from './dependency';
export function globalBridge() { boundary(); }
// Global augmentation bodies are also ambient module blocks.
declare global {
  import { safe as boundary } from './safe';
}
