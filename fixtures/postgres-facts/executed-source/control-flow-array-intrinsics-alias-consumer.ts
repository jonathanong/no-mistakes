import { read } from '@example/db';
import './control-flow-array-intrinsics-constructor-dependency';
import './control-flow-array-intrinsics-instance-dependency';
export function run() {
  read(['/* intrinsics/alias-imported */ SELECT 1'].join(''));
}
