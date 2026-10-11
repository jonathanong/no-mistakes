import { vi } from 'vitest';
const { setConfig: valid } = vi;
valid({ testTimeout: 30000, hookTimeout: 1 });
function unrelated(vi: { setConfig(value: unknown): void }) {
  const { setConfig: update } = vi;
  update({ testTimeout: 60000 });
}
let mutable = vi;
mutable = { setConfig() {} } as typeof vi;
mutable.setConfig({ testTimeout: 60000 });
const fake = { setConfig() {} };
const bound = fake.setConfig.bind(fake);
bound({ testTimeout: 60000 });
