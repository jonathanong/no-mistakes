import { mergeConfig as merge } from 'vitest/config';
export default merge({test:{testTimeout:30001}}, {test:{hookTimeout:1}});
