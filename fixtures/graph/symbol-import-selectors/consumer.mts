// Unused imports still count; an unrelated export must not erase them.
import primary, { other } from './source.mjs';
import { renamed } from './barrel.mjs';
import type { Contract } from './barrel.mjs';
export const unrelated = 0;
