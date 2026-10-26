import { wanted } from './ambiguous.mjs';
import missingDefault from './ambiguous.mjs';
import { absent } from './source.mjs';
import * as namespace from './source.mjs';
export const dynamic = () => import('./source.mjs');
export { absent as unresolved } from './missing.mjs';
