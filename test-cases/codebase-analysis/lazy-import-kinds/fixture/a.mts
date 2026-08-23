import type { Flag } from './b.mts';
const resolved = require.resolve('./b.mts');
import 'no-such-external-module-for-coverage';
export type { Flag };
void resolved;
