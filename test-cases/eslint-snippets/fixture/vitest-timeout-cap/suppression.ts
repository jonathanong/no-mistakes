import { it } from 'vitest';
// no-mistakes-disable-next-line vitest-timeout-cap
it('next', () => {}, 90000);
it('line', () => {}, 90000); // no-mistakes-disable-line no-mistakes/vitest-timeout-cap
// no-mistakes-disable-next-line other-rule
it('reported', () => {}, 90000);
const text = 'no-mistakes-disable-next-line vitest-timeout-cap';
it('reported', () => {}, 90000);
