import { bridge, arrow } from '../helpers/bridge';
import defaultBridge from '../helpers/bridge';
import { __test_1 as before } from '../helpers/collision-before';
import { __test_1 as after } from '../helpers/collision-after';
import { callSynthetic } from '../helpers/collision-call';
import { exportedSynthetic } from '../helpers/collision-export';
import { ambientBridge } from '../helpers/ambient';
import { callSyntheticUnescaped } from '../helpers/collision-call-unescaped';
describe('nested', () => {
  test('ambient imports', () => { ambientBridge(); });
  test('bridge', () => { bridge(); arrow(); defaultBridge(); });
  test('collision export', () => exportedSynthetic());
  test('collision call unescaped', () => callSyntheticUnescaped());
  test('collision call', () => callSynthetic());
  test('collision before', () => before());
  test('collision after', () => after());
});
