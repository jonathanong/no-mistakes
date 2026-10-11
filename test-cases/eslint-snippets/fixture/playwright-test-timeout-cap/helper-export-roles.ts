import { test, expect, register, ordinary } from './approved-test';
import * as helper from './approved-test';
expect(1).toBe(1); helper.expect(1).toBe(1);
ordinary(); const normal = () => {}; normal();
register('opaque wrapper', async () => {});
test('admitted registrar', async () => {});
helper.register('opaque namespace wrapper', async () => {});
