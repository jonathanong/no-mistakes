// Non-test importer of the target: proves the always-include glob does not
// turn an ordinary source file into a "test".
import { widget } from './widget.mts';

export const helper = () => widget();
