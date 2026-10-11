import {mergeConfig} from 'vitest/config';
export default mergeConfig({test:{testTimeout:30001}}, {}, chooseRootFlag());
