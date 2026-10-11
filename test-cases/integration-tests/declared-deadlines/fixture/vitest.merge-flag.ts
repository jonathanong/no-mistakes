import * as config from 'vitest/config';
export default config.mergeConfig({test:{testTimeout:30001}}, {test:{hookTimeout:1}}, false);
