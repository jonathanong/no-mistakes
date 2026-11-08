import { helper } from './helper.mjs';
export function dynamic(dispatch: () => void) { dispatch(); }
export function bounded() { helper(); }
export function mixed(dispatch: () => void) { dispatch(); helper(); }
