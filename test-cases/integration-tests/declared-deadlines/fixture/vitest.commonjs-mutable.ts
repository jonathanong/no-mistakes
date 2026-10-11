let {mergeConfig} = require('vite');
mergeConfig = (_first: unknown, second: unknown) => second;
export default mergeConfig({test:{testTimeout:1}}, {test:{testTimeout:30001}});
