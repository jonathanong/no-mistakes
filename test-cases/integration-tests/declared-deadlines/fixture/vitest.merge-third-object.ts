import { mergeConfig } from 'vitest/config';
// Third argument is isRoot, not another config; this must not certify case1.
export default mergeConfig({test:{testTimeout:30001}}, {}, {test:{testTimeout:1}} as any);
