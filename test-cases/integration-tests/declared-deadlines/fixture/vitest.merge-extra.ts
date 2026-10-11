import { mergeConfig } from 'vitest/config';
// Extra arguments are not merged by Vite. Never certify this final case1.
export default (mergeConfig as Function)({test:{testTimeout:30001}}, {}, true, {test:{testTimeout:1}});
