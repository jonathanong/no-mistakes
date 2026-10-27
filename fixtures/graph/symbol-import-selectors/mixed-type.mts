// Same source and physical line: the opaque module type must remain unknown.
import type { Contract } from './source.mjs'; export type Wrapped = Contract & typeof import('./source.mjs');
