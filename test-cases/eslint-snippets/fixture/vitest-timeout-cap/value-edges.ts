import { test, vi, beforeAll } from 'vitest';
import { defineConfig } from 'vitest/config';
const NEGATIVE = -1;
const POSITIVE = +60000;
const MIN = +30000;
const UNKNOWN = -Number(process.env.TIMEOUT);
const dynamicKey = process.env.KEY;
const getter = { get timeout() { return 60000; } };
const options = { timeout: POSITIVE };
const runtime = { testTimeout: POSITIVE };
test('negative within cap', () => {}, NEGATIVE);
test('positive', () => {}, POSITIVE);
test('boundary', () => {}, MIN);
test('unknown unary', () => {}, UNKNOWN);
test('unknown getter', getter, () => {});
// no-mistakes-disable-next-line vitest-timeout-cap -- suppress at options usage, not declaration
test('suppressed options', () => {}, options);
vi.setConfig(runtime);
function local() { const HIGH = 60000; beforeAll(() => {}, HIGH); }
const earlier = { testTimeout: POSITIVE, [dynamicKey]: 5000 };
const copied = { ...earlier };
export default defineConfig({ test: { ...copied, 'hookTimeout': POSITIVE } });
