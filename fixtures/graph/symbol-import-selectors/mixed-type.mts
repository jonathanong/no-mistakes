// Same source and physical line: the opaque type reference must remain unknown.
import type { Contract } from './source.mjs'; export type Wrapped = Contract & import('./source.mjs').Contract;
